use core::hash::BuildHasher;
use std::{
    collections::{BTreeSet, BinaryHeap, HashSet, LinkedList, VecDeque},
    sync::Arc,
};

use crate::{
    async_trait,
    io::{Write, WriteExt},
    Config, Endianness, Result,
};

#[async_trait]
/// Trait for encoding values
pub trait Encode {
    /// Returns size of encoded byte array
    fn size(&self) -> usize;

    /// Writes encoded byte array to writer and returns the number of bytes written
    ///
    /// ## Equivalent to:
    ///
    /// ```rust,ignore
    /// async fn encode_to<W>(&self, writer: W) -> Result<usize>
    /// where
    ///     W: Write + Unpin + Send
    /// ```
    async fn encode_to<W>(&self, config: &Config, writer: W) -> Result<usize>
    where
        W: Write + Unpin + Send;
}

macro_rules! impl_primitive {
    ($($type: tt),+) => {
        $(
            #[async_trait]
            impl Encode for $type {
                #[inline]
                fn size(&self) -> usize {
                    core::mem::size_of::<Self>()
                }

                async fn encode_to<W>(&self, config: &Config, mut writer: W) -> Result<usize>
                where
                    W: Write + Unpin + Send,
                {
                    match config.endianness {
                        Endianness::LittleEndian => writer.write(&self.to_le_bytes()).await.map_err(Into::into),
                        Endianness::BigEndian => writer.write(&self.to_be_bytes()).await.map_err(Into::into)
                    }
                }
            }
        )+
    };
}

impl_primitive!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize);

#[async_trait]
impl Encode for bool {
    #[inline]
    fn size(&self) -> usize {
        core::mem::size_of::<bool>()
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    async fn encode_to<W>(&self, config: &Config, writer: W) -> Result<usize>
    where
        W: Write + Unpin + Send,
    {
        (*self as u8).encode_to(config, writer).await
    }
}

#[async_trait]
impl Encode for char {
    #[inline]
    fn size(&self) -> usize {
        core::mem::size_of::<char>()
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    async fn encode_to<W>(&self, config: &Config, writer: W) -> Result<usize>
    where
        W: Write + Unpin + Send,
    {
        (*self as u32).encode_to(config, writer).await
    }
}

#[async_trait]
impl<T> Encode for Option<T>
where
    T: Encode + Sync,
{
    fn size(&self) -> usize {
        match self {
            Some(ref value) => core::mem::size_of::<u8>() + value.size(),
            None => core::mem::size_of::<u8>(),
        }
    }

    async fn encode_to<W>(&self, config: &Config, mut writer: W) -> Result<usize>
    where
        W: Write + Unpin + Send,
    {
        match self {
            None => 0u8.encode_to(config, &mut writer).await.map_err(Into::into),
            Some(ref value) => Ok(1u8.encode_to(config, &mut writer).await?
                + value.encode_to(config, &mut writer).await?),
        }
    }
}

#[async_trait]
impl<T, E> Encode for core::result::Result<T, E>
where
    T: Encode + Sync,
    E: Encode + Sync,
{
    fn size(&self) -> usize {
        match self {
            Ok(ref value) => core::mem::size_of::<u8>() + value.size(),
            Err(ref err) => core::mem::size_of::<u8>() + err.size(),
        }
    }

    async fn encode_to<W>(&self, config: &Config, mut writer: W) -> Result<usize>
    where
        W: Write + Unpin + Send,
    {
        match self {
            Ok(ref value) => {
                Ok(0u8.encode_to(config, &mut writer).await?
                    + value.encode_to(config, writer).await?)
            }
            Err(ref err) => {
                Ok(1u8.encode_to(config, &mut writer).await?
                    + err.encode_to(config, writer).await?)
            }
        }
    }
}

macro_rules! impl_seq {
    ($ty: tt < T $(: $tbound1: tt $(+ $tbound2: ident)*)* $(, $typaram: tt : $bound1: tt $(+ $bound2: tt)*)* >) => {
        #[async_trait]
        impl<T $(, $typaram)*> Encode for $ty<T $(, $typaram)*>
        where
            T: Encode + Sync $(+ $tbound1 $(+ $tbound2)*)*,
            $($typaram: $bound1 $(+ $bound2)*,)*
        {
            #[inline]
            fn size(&self) -> usize {
                core::mem::size_of::<u64>() + self.iter().map(Encode::size).sum::<usize>()
            }

            #[allow(clippy::ptr_arg)]
            async fn encode_to<W>(&self, config: &Config, mut writer: W) -> Result<usize>
            where
                W: Write + Unpin + Send,
            {
                let mut encoded = 0;

                encoded += (self.len() as u64).encode_to(config, &mut writer).await?;

                for item in self.iter() {
                    encoded += item.encode_to(config, &mut writer).await?;
                }

                Ok(encoded)
            }
        }
    };
}

impl_seq!(Vec<T>);
impl_seq!(VecDeque<T>);
impl_seq!(LinkedList<T>);
impl_seq!(HashSet<T, S: BuildHasher + Sync>);
impl_seq!(BTreeSet<T: 'static>);
impl_seq!(BinaryHeap<T>);

#[async_trait]
impl<T> Encode for [T]
where
    T: Encode + Sync,
{
    #[inline]
    fn size(&self) -> usize {
        core::mem::size_of::<u64>() + self.iter().map(Encode::size).sum::<usize>()
    }

    async fn encode_to<W>(&self, config: &Config, mut writer: W) -> Result<usize>
    where
        W: Write + Unpin + Send,
    {
        let mut encoded = 0;

        encoded += (self.len() as u64).encode_to(config, &mut writer).await?;

        for item in self.iter() {
            encoded += item.encode_to(config, &mut writer).await?;
        }

        Ok(encoded)
    }
}

#[async_trait]
impl Encode for String {
    #[inline]
    fn size(&self) -> usize {
        <str>::size(self)
    }

    #[allow(clippy::ptr_arg)]
    async fn encode_to<W>(&self, config: &Config, writer: W) -> Result<usize>
    where
        W: Write + Unpin + Send,
    {
        <str>::encode_to(self, config, writer).await
    }
}

#[async_trait]
impl Encode for str {
    #[inline]
    fn size(&self) -> usize {
        core::mem::size_of::<u64>() + self.len()
    }

    async fn encode_to<W>(&self, config: &Config, writer: W) -> Result<usize>
    where
        W: Write + Unpin + Send,
    {
        self.as_bytes().encode_to(config, writer).await
    }
}

macro_rules! impl_deref {
    ($($desc: tt)+) => {
        #[async_trait]
        impl $($desc)+ {
            #[inline]
            fn size(&self) -> usize {
                <T>::size(self)
            }

            async fn encode_to<W>(&self, config: &Config, writer: W) -> Result<usize>
            where
                W: Write + Unpin + Send,
            {
                <T>::encode_to(self, config, writer).await
            }
        }
    }
}

impl_deref!(<T: ?Sized> Encode for &T where T: Encode + Sync);
impl_deref!(<T: ?Sized> Encode for &mut T where T: Encode + Sync);
impl_deref!(<T: ?Sized> Encode for Box<T> where T: Encode + Sync);
impl_deref!(<T: ?Sized> Encode for Arc<T> where T: Encode + Sync + Send);

macro_rules! impl_fixed_arr {
    ($($len: tt),+) => {
        $(
            #[async_trait]
            impl<T> Encode for [T; $len]
            where
                T: Encode + Sync,
            {
                #[inline]
                fn size(&self) -> usize {
                    self.iter().map(Encode::size).sum::<usize>()
                }

                async fn encode_to<W>(&self, config: &Config, mut writer: W) -> Result<usize>
                where
                    W: Write + Unpin + Send,
                {
                    let mut encoded = 0;

                    for item in self.iter() {
                        encoded += item.encode_to(config, &mut writer).await?;
                    }

                    Ok(encoded)
                }
            }
        )+
    };
}

impl_fixed_arr!(
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50,
    51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 128, 256, 512, 1024
);

#[async_trait]
impl<A, B> Encode for (A, B)
where
    A: Encode + Send + Sync,
    B: Encode + Send + Sync,
{
    #[inline]
    fn size(&self) -> usize {
        self.0.size() + self.1.size()
    }

    async fn encode_to<W>(&self, config: &Config, mut writer: W) -> Result<usize>
    where
        W: Write + Unpin + Send,
    {
        let mut encoded = 0;

        encoded += self.0.encode_to(config, &mut writer).await?;
        encoded += self.1.encode_to(config, &mut writer).await?;

        Ok(encoded)
    }
}
