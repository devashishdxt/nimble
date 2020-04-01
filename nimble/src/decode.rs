use core::{
    convert::TryFrom,
    hash::{BuildHasher, Hash},
    marker::PhantomData,
    num::{
        NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
        NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
    },
};
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque},
    ffi::CString,
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

impl_primitive!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize, f32, f64);

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

macro_rules! impl_from_bytes {
    ($type: ty, $create: ident) => {
        #[async_trait]
        impl Decode for $type {
            async fn decode_from<R>(config: &Config, reader: R) -> Result<Self>
            where
                R: Read + Unpin + Send,
            {
                let bytes = <Vec<u8>>::decode_from(config, reader).await?;
                Self::$create(bytes).map_err(Into::into)
            }
        }
    };
}

impl_from_bytes!(String, from_utf8);
impl_from_bytes!(CString, new);

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

#[async_trait]
impl<'a, T: ?Sized> Decode for Cow<'a, T>
where
    T: 'a + ToOwned,
    <T as ToOwned>::Owned: Decode,
{
    async fn decode_from<R>(config: &Config, reader: R) -> Result<Self>
    where
        R: Read + Unpin + Send,
    {
        let owned = <<T as ToOwned>::Owned>::decode_from(config, reader).await?;
        Ok(Cow::Owned(owned))
    }
}

macro_rules! impl_map {
    (
        $ty: ident < K $(: $kbound1: ident $(+ $kbound2: ident)*)*, V $(, $typaram: ident : $bound1: ident $(+ $bound2: ident)*)* >,
        $len: ident,
        $create: expr
    ) => {
        #[async_trait]
        impl<K, V $(, $typaram)*> Decode for $ty<K, V $(, $typaram)*>
        where
            K: Decode + Send $(+ $kbound1 $(+ $kbound2)*)*,
            V: Decode + Send,
            $($typaram: $bound1 $(+ $bound2)*,)*
        {
            async fn decode_from<R>(config: &Config, mut reader: R) -> Result<Self>
            where
                R: Read + Unpin + Send,
            {
                let $len = u64::decode_from(config, &mut reader).await?;
                let $len = usize::try_from($len).map_err(|_| Error::InvalidLength($len))?;

                let mut map = $create;

                for _ in 0..$len {
                    let entry = <(K, V)>::decode_from(config, &mut reader).await?;
                    map.insert(entry.0, entry.1);
                }

                Ok(map)
            }
        }
    };
}

impl_map!(
    HashMap<K: Eq + Hash, V, S: BuildHasher + Default + Send>,
    len,
    HashMap::with_capacity_and_hasher(len, S::default())
);
impl_map!(BTreeMap<K: Ord, V>, len, BTreeMap::new());

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
impl Decode for () {
    async fn decode_from<R>(_config: &Config, _reader: R) -> Result<Self>
    where
        R: Read + Unpin + Send,
    {
        Ok(())
    }
}

macro_rules! impl_tuple {
    ($(($($name:tt)+))+) => {
        $(
            #[async_trait]
            impl<$($name),+> Decode for ($($name,)+)
            where
                $($name: Decode + Send,)+
            {
                async fn decode_from<R>(config: &Config, mut reader: R) -> Result<Self>
                where
                    R: Read + Unpin + Send,
                {
                    Ok((
                        $(
                            $name::decode_from(&config, &mut reader).await?,
                        )+
                    ))
                }
            }
        )+
    }
}

impl_tuple! {
    (T0)
    (T0 T1)
    (T0 T1 T2)
    (T0 T1 T2 T3)
    (T0 T1 T2 T3 T4)
    (T0 T1 T2 T3 T4 T5)
    (T0 T1 T2 T3 T4 T5 T6)
    (T0 T1 T2 T3 T4 T5 T6 T7)
    (T0 T1 T2 T3 T4 T5 T6 T7 T8)
    (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9)
    (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10)
    (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11)
    (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12)
    (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13)
    (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14)
    (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15)
}

#[async_trait]
impl<T> Decode for PhantomData<T>
where
    T: ?Sized,
{
    async fn decode_from<R>(_config: &Config, _reader: R) -> Result<Self>
    where
        R: Read + Unpin + Send,
    {
        Ok(Default::default())
    }
}

macro_rules! impl_non_zero_primitives {
    ($($type: ident),+) => {
        $(
            #[async_trait]
            impl Decode for $type {
                async fn decode_from<R>(config: &Config, reader: R) -> Result<Self>
                where
                    R: Read + Unpin + Send,
                {
                    Ok(Self::new(Decode::decode_from(config, reader).await?)
                        .ok_or_else(|| Error::NonZeroError)?)
                }
            }
        )+
    };
}

impl_non_zero_primitives!(
    NonZeroU8,
    NonZeroU16,
    NonZeroU32,
    NonZeroU64,
    NonZeroU128,
    NonZeroI8,
    NonZeroI16,
    NonZeroI32,
    NonZeroI64,
    NonZeroI128,
    NonZeroUsize,
    NonZeroIsize
);
