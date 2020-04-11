//! Utilities for encoding/decoding VarInt
use core::{convert::TryFrom, mem::size_of};

use async_trait::async_trait;

use crate::{
    io::{Read, Write},
    Config, Decode, Encode, Error, Result,
};

/// Base 128 VarInt ([Reference](https://developers.google.com/protocol-buffers/docs/encoding#varints))
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct VarInt(u128);

#[async_trait]
impl Encode for VarInt {
    fn size(&self) -> usize {
        if self.0 == 0 {
            return 1;
        }

        let mut counter = 0;
        let mut num = self.0;

        while num > 0 {
            counter += 1;
            num >>= 7;
        }

        counter
    }

    async fn encode_to<W>(&self, config: &Config, mut writer: W) -> Result<usize>
    where
        W: Write + Unpin + Send,
    {
        let mut num = self.0;
        let mut encoded = 0;

        while num >= 0b1000_0000 {
            let byte: u8 = (num & 0b0111_1111) as u8 | 0b1000_0000;
            encoded += byte.encode_to(config, &mut writer).await?;

            num >>= 7;
        }

        encoded += (num as u8).encode_to(config, &mut writer).await?;

        Ok(encoded)
    }
}

#[async_trait]
impl Decode for VarInt {
    async fn decode_from<R>(config: &Config, mut reader: R) -> Result<Self>
    where
        R: Read + Unpin + Send,
    {
        let mut num: u128 = 0;
        let mut shift_by: u8 = 0;

        loop {
            let byte = u8::decode_from(config, &mut reader).await?;
            num |= ((byte & 0b0111_1111) as u128) << shift_by;

            let has_next_byte = byte & 0b1000_0000 != 0;

            if has_next_byte {
                shift_by += 7;
            } else {
                break;
            }
        }

        Ok(VarInt(num))
    }
}

macro_rules! impl_from_non_zigzagged_to_varint {
    ($($type: ty),+) => {
        $(
            impl From<$type> for VarInt {
                fn from(num: $type) -> Self {
                    Self(num.into())
                }
            }
        )+
    };
}

macro_rules! impl_from_zigzagged_to_varint {
    ($($type: ty),+) => {
        $(
            impl From<$type> for VarInt {
                fn from(num: $type) -> Self {
                    Self(num.encode_zigzag().into())
                }
            }
        )+
    };
}

impl TryFrom<usize> for VarInt {
    type Error = Error;

    fn try_from(num: usize) -> Result<Self> {
        Ok(Self(TryFrom::try_from(num)?))
    }
}

impl TryFrom<isize> for VarInt {
    type Error = Error;

    fn try_from(num: isize) -> Result<Self> {
        Ok(i128::try_from(num)?.into())
    }
}

impl_from_non_zigzagged_to_varint!(u8, u16, u32, u64, u128);
impl_from_zigzagged_to_varint!(i8, i16, i32, i64, i128);

macro_rules! impl_from_varint_to_non_zigzagged {
    ($($type: ty),+) => {
        $(
            impl TryFrom<VarInt> for $type {
                type Error = Error;

                fn try_from(value: VarInt) -> Result<Self> {
                    Self::try_from(value.0).map_err(Into::into)
                }
            }
        )+
    };
}

macro_rules! impl_from_varint_to_zigzagged {
    ($($type: ty),+) => {
        $(
            impl TryFrom<VarInt> for $type {
                type Error = Error;

                fn try_from(value: VarInt) -> Result<Self> {
                    Ok(<$type>::decode_zigzag(TryFrom::try_from(value.0)?))
                }
            }
        )+
    };
}

// This implementation does not make sense. This is here just for consistency.
impl TryFrom<VarInt> for u128 {
    type Error = Error;

    fn try_from(value: VarInt) -> Result<Self> {
        Ok(value.0)
    }
}

// This implementation does not make sense. This is here just for consistency.
impl TryFrom<VarInt> for i128 {
    type Error = Error;

    fn try_from(value: VarInt) -> Result<Self> {
        Ok(i128::decode_zigzag(value.0))
    }
}

impl_from_varint_to_non_zigzagged!(u8, u16, u32, u64, usize);
impl_from_varint_to_zigzagged!(i8, i16, i32, i64, isize);

/// Type to encode/decode value to/from zigzagged format
pub trait ZigZag {
    /// Type of zigzagged value
    type Output;

    /// Encodes the value in zigzagged format
    fn encode_zigzag(self) -> Self::Output;

    /// Decoded a value from zigzagged format
    fn decode_zigzag(encoded: Self::Output) -> Self;
}

macro_rules! impl_zigzag {
    ($(($type: ty => $output: ty)),+) => {
        $(
            impl ZigZag for $type {
                type Output = $output;

                fn encode_zigzag(self) -> Self::Output {
                    ((self << 1) ^ (self >> (size_of::<Self>() * 8 - 1))) as Self::Output
                }

                fn decode_zigzag(encoded: Self::Output) -> Self {
                    ((encoded >> 1) ^ (-(encoded as Self & 1)) as Self::Output) as Self
                }
            }
        )+
    };
}

impl_zigzag!((i8 => u8), (i16 => u16), (i32 => u32), (i64 => u64), (i128 => u128), (isize => usize));

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! impl_zigzag_test {
        ($type: ty => $output: ty, $name: ident) => {
            #[test]
            fn $name() {
                assert_eq!(0, (0 as $type).encode_zigzag());
                assert_eq!(1, (-1 as $type).encode_zigzag());
                assert_eq!(2, (1 as $type).encode_zigzag());
                assert_eq!(3, (-2 as $type).encode_zigzag());
                assert_eq!(4, (2 as $type).encode_zigzag());
                assert_eq!(
                    <$output>::max_value() - 2,
                    (<$type>::min_value() + 1).encode_zigzag()
                );
                assert_eq!(
                    <$output>::max_value() - 1,
                    <$type>::max_value().encode_zigzag()
                );
                assert_eq!(<$output>::max_value(), <$type>::min_value().encode_zigzag());

                assert_eq!(0, <$type>::decode_zigzag(0));
                assert_eq!(-1, <$type>::decode_zigzag(1));
                assert_eq!(1, <$type>::decode_zigzag(2));
                assert_eq!(-2, <$type>::decode_zigzag(3));
                assert_eq!(2, <$type>::decode_zigzag(4));
                assert_eq!(
                    (<$type>::min_value() + 1),
                    <$type>::decode_zigzag(<$output>::max_value() - 2)
                );
                assert_eq!(
                    <$type>::max_value(),
                    <$type>::decode_zigzag(<$output>::max_value() - 1),
                );
                assert_eq!(
                    <$type>::min_value(),
                    <$type>::decode_zigzag(<$output>::max_value())
                );
            }
        };
    }

    impl_zigzag_test!(i8 => u8, zigzag_i8_test);
    impl_zigzag_test!(i16 => u16, zigzag_i16_test);
    impl_zigzag_test!(i32 => u32, zigzag_i32_test);
    impl_zigzag_test!(i64 => u64, zigzag_i64_test);
    impl_zigzag_test!(i128 => u128, zigzag_i128_test);
    impl_zigzag_test!(isize => usize, zigzag_isize_test);
}
