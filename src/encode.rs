#[cfg(feature = "async-std")]
use async_std::io::{prelude::WriteExt, Write};

#[cfg(feature = "tokio")]
use tokio::io::{AsyncWrite as Write, AsyncWriteExt};

use core::{future::Future, pin::Pin};

use crate::error::Result;

pub trait Encode {
    fn size(&self) -> usize;

    fn encode_to<'a, 't, W>(
        &'a self,
        writer: W,
    ) -> Pin<Box<dyn Future<Output = Result<usize>> + Send + 't>>
    where
        W: Write + Unpin + Send + 't,
        'a: 't,
        Self: 't;
}

macro_rules! impl_primitive {
    ($($type: ty),+) => {
        $(
            impl Encode for $type {
                #[inline]
                fn size(&self) -> usize {
                    core::mem::size_of::<Self>()
                }

                fn encode_to<'a, 't, W>(
                    &'a self,
                    writer: W,
                ) -> Pin<Box<dyn Future<Output = Result<usize>> + Send + 't>>
                where
                    W: Write + Unpin + Send + 't,
                    'a: 't,
                    Self: 't,
                {
                    async fn __encode_to<W>(_self: $type, mut writer: W) -> Result<usize>
                    where
                        W: Write + Unpin + Send,
                    {
                        writer.write(&_self.to_be_bytes()).await.map_err(Into::into)
                    }

                    Box::pin(__encode_to::<W>(*self, writer))
                }
            }
        )+
    };
}

impl_primitive!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize);

impl Encode for bool {
    #[inline]
    fn size(&self) -> usize {
        core::mem::size_of::<bool>()
    }

    fn encode_to<'a, 't, W>(
        &'a self,
        writer: W,
    ) -> Pin<Box<dyn Future<Output = Result<usize>> + Send + 't>>
    where
        W: Write + Unpin + Send + 't,
        'a: 't,
        Self: 't,
    {
        async fn __encode_to<I>(_self: bool, writer: I) -> Result<usize>
        where
            I: Write + Unpin + Send,
        {
            (_self as u8).encode_to(writer).await
        }

        Box::pin(__encode_to(*self, writer))
    }
}

impl Encode for char {
    #[inline]
    fn size(&self) -> usize {
        core::mem::size_of::<char>()
    }

    fn encode_to<'a, 't, W>(
        &'a self,
        writer: W,
    ) -> Pin<Box<dyn Future<Output = Result<usize>> + Send + 't>>
    where
        W: Write + Unpin + Send + 't,
        'a: 't,
        Self: 't,
    {
        async fn __encode_to<I>(_self: char, writer: I) -> Result<usize>
        where
            I: Write + Unpin + Send,
        {
            (_self as u32).encode_to(writer).await
        }

        Box::pin(__encode_to(*self, writer))
    }
}

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

    fn encode_to<'a, 't, W>(
        &'a self,
        writer: W,
    ) -> Pin<Box<dyn Future<Output = Result<usize>> + Send + 't>>
    where
        W: Write + Unpin + Send + 't,
        'a: 't,
        Self: 't,
    {
        async fn __encode_to<T, I>(_self: &Option<T>, mut writer: I) -> Result<usize>
        where
            T: Encode,
            I: Write + Unpin + Send,
        {
            match _self {
                None => writer.write(&[0]).await.map_err(Into::into),
                Some(ref value) => Ok(writer.write(&[1]).await? + value.encode_to(writer).await?),
            }
        }

        Box::pin(__encode_to(self, writer))
    }
}

impl<T> Encode for Vec<T>
where
    T: Encode + Sync,
{
    #[inline]
    fn size(&self) -> usize {
        <[T]>::size(self)
    }

    fn encode_to<'a, 't, W>(
        &'a self,
        writer: W,
    ) -> Pin<Box<dyn Future<Output = Result<usize>> + Send + 't>>
    where
        W: Write + Unpin + Send + 't,
        'a: 't,
        Self: 't,
    {
        <[T]>::encode_to(self, writer)
    }
}

impl<T> Encode for [T]
where
    T: Encode + Sync,
{
    #[inline]
    fn size(&self) -> usize {
        core::mem::size_of::<u64>() + self.iter().map(Encode::size).sum::<usize>()
    }

    fn encode_to<'a, 't, W>(
        &'a self,
        writer: W,
    ) -> Pin<Box<dyn Future<Output = Result<usize>> + Send + 't>>
    where
        W: Write + Unpin + Send + 't,
        'a: 't,
        Self: 't,
    {
        async fn __encode_to<T, I>(_self: &[T], mut writer: I) -> Result<usize>
        where
            T: Encode,
            I: Write + Unpin + Send,
        {
            let mut encoded = 0;

            encoded += (_self.len() as u64).encode_to(&mut writer).await?;

            for item in _self.iter() {
                encoded += item.encode_to(&mut writer).await?;
            }

            Ok(encoded)
        }

        Box::pin(__encode_to(self, writer))
    }
}

impl Encode for String {
    #[inline]
    fn size(&self) -> usize {
        <str>::size(self)
    }

    #[inline]
    fn encode_to<'a, 't, W>(
        &'a self,
        writer: W,
    ) -> Pin<Box<dyn Future<Output = Result<usize>> + Send + 't>>
    where
        W: Write + Unpin + Send + 't,
        'a: 't,
        Self: 't,
    {
        <str>::encode_to(self, writer)
    }
}

impl Encode for str {
    #[inline]
    fn size(&self) -> usize {
        core::mem::size_of::<u64>() + self.len()
    }

    #[inline]
    fn encode_to<'a, 't, W>(
        &'a self,
        writer: W,
    ) -> Pin<Box<dyn Future<Output = Result<usize>> + Send + 't>>
    where
        W: Write + Unpin + Send + 't,
        'a: 't,
        Self: 't,
    {
        self.as_bytes().encode_to(writer)
    }
}

macro_rules! impl_fixed_arr {
    ($($len: expr),+) => {
        $(
            impl<T> Encode for [T; $len]
            where
                T: Encode + Sync,
            {
                #[inline]
                fn size(&self) -> usize {
                    self.iter().map(Encode::size).sum::<usize>()
                }

                #[inline]
                fn encode_to<'a, 't, W>(
                    &'a self,
                    writer: W,
                ) -> Pin<Box<dyn Future<Output = Result<usize>> + Send + 't>>
                where
                    W: Write + Unpin + Send + 't,
                    'a: 't,
                    Self: 't,
                {
                    async fn __encode_to<T, I>(_self: &[T; $len], mut writer: I) -> Result<usize>
                    where
                        T: Encode,
                        I: Write + Unpin + Send,
                    {
                        let mut encoded = 0;

                        for item in _self.iter() {
                            encoded += item.encode_to(&mut writer).await?;
                        }

                        Ok(encoded)
                    }

                    Box::pin(__encode_to(self, writer))
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
