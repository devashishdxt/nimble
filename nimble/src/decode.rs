use core::{
    convert::TryFrom,
    hash::{BuildHasher, Hash},
};
use std::{
    collections::{BTreeSet, BinaryHeap, HashSet, LinkedList, VecDeque},
    rc::Rc,
    sync::Arc,
};

use arrayvec::ArrayVec;

use crate::{
    async_trait,
    io::{Read, ReadExt},
    Config, Endianness, Error, Result,
};

#[async_trait]
/// Trait for decoding values
pub trait Decode: Sized {
    /// Decodes values from reader
    ///
    /// ## Equivalent to:
    ///
    /// ```rust,ignore
    /// async fn decode_from<R>(reader: R) -> Result<Self>
    /// where
    ///     R: Read + Unpin + Send
    /// ```
    async fn decode_from<R>(config: &Config, reader: R) -> Result<Self>
    where
        R: Read + Unpin + Send;
}

macro_rules! impl_primitive {
    ($($type: ty),+) => {
        $(
            #[async_trait]
            impl Decode for $type {
                async fn decode_from<R>(config: &Config, mut reader: R) -> Result<Self>
                where
                    R: Read + Unpin + Send
                {
                    let mut bytes = [0u8; core::mem::size_of::<$type>()];
                    reader.read_exact(&mut bytes).await?;

                    match config.endianness {
                        Endianness::LittleEndian => {
                            Ok(<$type>::from_le_bytes(bytes))
                        }
                        Endianness::BigEndian => {
                            Ok(<$type>::from_be_bytes(bytes))
                        }
                    }
                }
            }
        )+
    };
}

impl_primitive!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize);

#[async_trait]
impl Decode for bool {
    async fn decode_from<R>(config: &Config, reader: R) -> Result<Self>
    where
        R: Read + Unpin + Send,
    {
        Ok(<u8>::decode_from(config, reader).await? != 0)
    }
}

#[async_trait]
impl Decode for char {
    async fn decode_from<R>(config: &Config, reader: R) -> Result<Self>
    where
        R: Read + Unpin + Send,
    {
        let code = <u32>::decode_from(config, reader).await?;
        core::char::from_u32(code).ok_or_else(|| Error::InvalidChar(code))
    }
}

#[async_trait]
impl<T> Decode for Option<T>
where
    T: Decode,
{
    async fn decode_from<R>(config: &Config, mut reader: R) -> Result<Self>
    where
        R: Read + Unpin + Send,
    {
        let option = u8::decode_from(config, &mut reader).await?;

        match option {
            0 => Ok(None),
            1 => T::decode_from(config, &mut reader).await.map(Some),
            _ => Err(Error::InvalidEnumVariant(option as u32)),
        }
    }
}

#[async_trait]
impl<T, E> Decode for core::result::Result<T, E>
where
    T: Decode,
    E: Decode,
{
    async fn decode_from<R>(config: &Config, mut reader: R) -> Result<Self>
    where
        R: Read + Unpin + Send,
    {
        let option = u8::decode_from(config, &mut reader).await?;

        match option {
            0 => T::decode_from(config, &mut reader).await.map(Ok),
            1 => E::decode_from(config, &mut reader).await.map(Err),
            _ => Err(Error::InvalidEnumVariant(option as u32)),
        }
    }
}

macro_rules! impl_seq {
    (
        $ty: ident < T $(: $tbound1: ident $(+ $tbound2: ident)*)* $(, $typaram: ident : $bound1: ident $(+ $bound2: ident)*)* >,
        $len: ident,
        $create: expr,
        $insert: expr
    ) => {
        #[async_trait]
        impl<T $(, $typaram)*> Decode for $ty<T $(, $typaram)*>
        where
            T: Decode + Send $(+ $tbound1 $(+ $tbound2)*)*,
            $($typaram: $bound1 $(+ $bound2)*,)*
        {
            async fn decode_from<R>(config: &Config, mut reader: R) -> Result<Self>
            where
                R: Read + Unpin + Send,
            {
                let $len = u64::decode_from(config, &mut reader).await?;
                let $len = usize::try_from($len).map_err(|_| Error::InvalidLength($len))?;

                let mut value = $create;

                for _ in 0..$len {
                    $insert(&mut value, T::decode_from(config, &mut reader).await?);
                }

                Ok(value)
            }
        }
    };
}

impl_seq!(Vec<T>, len, Vec::with_capacity(len), Vec::push);
impl_seq!(
    VecDeque<T>,
    len,
    VecDeque::with_capacity(len),
    VecDeque::push_back
);
impl_seq!(LinkedList<T>, len, LinkedList::new(), LinkedList::push_back);
impl_seq!(
    HashSet<T: Eq + Hash, S: BuildHasher + Default + Send>,
    len,
    HashSet::with_capacity_and_hasher(len, S::default()),
    HashSet::insert
);
impl_seq!(BTreeSet<T: Ord>, len, BTreeSet::new(), BTreeSet::insert);
impl_seq!(BinaryHeap<T: Ord>, len, BinaryHeap::new(), BinaryHeap::push);

#[async_trait]
impl Decode for String {
    async fn decode_from<R>(config: &Config, reader: R) -> Result<Self>
    where
        R: Read + Unpin + Send,
    {
        let bytes = <Vec<u8>>::decode_from(config, reader).await?;
        String::from_utf8(bytes).map_err(Into::into)
    }
}

macro_rules! impl_deref {
    ($type: ty, $func: expr) => {
        #[async_trait]
        impl<T> Decode for $type
        where
            T: Decode,
        {
            async fn decode_from<R>(config: &Config, reader: R) -> Result<Self>
            where
                R: Read + Unpin + Send,
            {
                T::decode_from(config, reader).await.map($func)
            }
        }
    };
}

impl_deref!(Box<T>, Box::new);
impl_deref!(Rc<T>, Rc::new);
impl_deref!(Arc<T>, Arc::new);

macro_rules! impl_fixed_arr {
    ($($len: expr),+) => {
        $(
            #[async_trait]
            impl<T> Decode for [T; $len]
            where
                T: Decode + Send,
            {
                async fn decode_from<R>(config: &Config, mut reader: R) -> Result<Self>
                where
                    R: Read + Unpin + Send,
                {
                    let mut arr = ArrayVec::<[T; $len]>::new();

                    for _ in 0..$len {
                        let value = T::decode_from(config, &mut reader).await?;
                        arr.push(value)
                    }

                    arr.into_inner().map_err(|_| Error::PartiallyFilledArray)
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
impl<A, B> Decode for (A, B)
where
    A: Decode + Send,
    B: Decode + Send,
{
    async fn decode_from<R>(config: &Config, mut reader: R) -> Result<Self>
    where
        R: Read + Unpin + Send,
    {
        Ok((
            A::decode_from(&config, &mut reader).await?,
            B::decode_from(&config, &mut reader).await?,
        ))
    }
}
