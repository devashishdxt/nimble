use std::sync::Arc;

use crate::{
    async_trait,
    error::Result,
    io::{Write, WriteExt},
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
    async fn encode_to<W>(&self, writer: W) -> Result<usize>
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

                #[cfg(feature = "little-endian")]
                async fn encode_to<W>(&self, mut writer: W) -> Result<usize>
                where
                    W: Write + Unpin + Send,
                {
                    writer.write(&self.to_le_bytes()).await.map_err(Into::into)
                }

                #[cfg(feature = "big-endian")]
                async fn encode_to<W>(&self, mut writer: W) -> Result<usize>
                where
                    W: Write + Unpin + Send,
                {
                    writer.write(&self.to_be_bytes()).await.map_err(Into::into)
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
    async fn encode_to<W>(&self, writer: W) -> Result<usize>
    where
        W: Write + Unpin + Send,
    {
        (*self as u8).encode_to(writer).await
    }
}

#[async_trait]
impl Encode for char {
    #[inline]
    fn size(&self) -> usize {
        core::mem::size_of::<char>()
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    async fn encode_to<W>(&self, writer: W) -> Result<usize>
    where
        W: Write + Unpin + Send,
    {
        (*self as u32).encode_to(writer).await
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

    async fn encode_to<W>(&self, mut writer: W) -> Result<usize>
    where
        W: Write + Unpin + Send,
    {
        match self {
            None => writer.write(&[0]).await.map_err(Into::into),
            Some(ref value) => Ok(writer.write(&[1]).await? + value.encode_to(writer).await?),
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

    async fn encode_to<W>(&self, mut writer: W) -> Result<usize>
    where
        W: Write + Unpin + Send,
    {
        match _self {
            Ok(ref value) => Ok(writer.write(&[0]).await? + value.encode_to(writer).await?),
            Err(ref err) => Ok(writer.write(&[1]).await? + err.encode_to(writer).await?),
        }
    }
}

#[async_trait]
impl<T> Encode for Vec<T>
where
    T: Encode + Sync,
{
    #[inline]
    fn size(&self) -> usize {
        <[T]>::size(self)
    }

    #[allow(clippy::ptr_arg)]
    async fn encode_to<W>(&self, writer: W) -> Result<usize>
    where
        W: Write + Unpin + Send,
    {
        <[T]>::encode_to(self, writer).await
    }
}

#[async_trait]
impl<T> Encode for [T]
where
    T: Encode + Sync,
{
    #[inline]
    fn size(&self) -> usize {
        core::mem::size_of::<u64>() + self.iter().map(Encode::size).sum::<usize>()
    }

    async fn encode_to<W>(&self, mut writer: W) -> Result<usize>
    where
        W: Write + Unpin + Send,
    {
        let mut encoded = 0;

        encoded += (_self.len() as u64).encode_to(&mut writer).await?;

        for item in _self.iter() {
            encoded += item.encode_to(&mut writer).await?;
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
    async fn encode_to<W>(&self, writer: W) -> Result<usize>
    where
        W: Write + Unpin + Send,
    {
        <str>::encode_to(self, writer).await
    }
}

#[async_trait]
impl Encode for str {
    #[inline]
    fn size(&self) -> usize {
        core::mem::size_of::<u64>() + self.len()
    }

    async fn encode_to<W>(&self, writer: W) -> Result<usize>
    where
        W: Write + Unpin + Send,
    {
        self.as_bytes().encode_to(writer).await
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

            async fn encode_to<W>(&self, writer: W) -> Result<usize>
            where
                W: Write + Unpin + Send,
            {
                <T>::encode_to(self, writer).await
            }
        }
    }
}

impl_deref!(<T: ?Sized> Encode for &T where T: Encode + Sync);
impl_deref!(<T: ?Sized> Encode for &mut T where T: Encode + Sync);
impl_deref!(<T: ?Sized> Encode for Box<T> where T: Encode + Sync);
impl_deref!(<T: ?Sized> Encode for Arc<T> where T: Encode + Sync + Send);
// impl_deref!(<T: ?Sized> Encode for Cow<'_, T> where T: Encode + ToOwned + Sync, <T as ToOwned>::Owned: Sync);

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

                async fn encode_to<W>(&self, mut writer: W) -> Result<usize>
                where
                    W: Write + Unpin + Send,
                {
                    let mut encoded = 0;

                    for item in _self.iter() {
                        encoded += item.encode_to(&mut writer).await?;
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
