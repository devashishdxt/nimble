use std::io::{Result, Write};

pub trait Encode {
    fn size(&self) -> usize;

    fn encode(&self) -> Vec<u8>;

    fn encode_to<W: Write>(&self, writer: W) -> Result<usize>;
}

macro_rules! impl_primitive {
    ($($type: ty),+) => {
        $(
            impl Encode for $type {
                #[inline]
                fn size(&self) -> usize {
                    std::mem::size_of::<Self>()
                }

                #[inline]
                fn encode(&self) -> Vec<u8> {
                    self.to_be_bytes().to_vec()
                }

                #[inline]
                fn encode_to<W: Write>(&self, mut writer: W) -> Result<usize> {
                    writer.write(&self.to_be_bytes())
                }
            }
        )+
    };
}

impl_primitive!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize);

impl Encode for bool {
    #[inline]
    fn size(&self) -> usize {
        std::mem::size_of::<bool>()
    }

    #[inline]
    fn encode(&self) -> Vec<u8> {
        (*self as u8).encode()
    }

    #[inline]
    fn encode_to<W: Write>(&self, writer: W) -> Result<usize> {
        (*self as u8).encode_to(writer)
    }
}

impl Encode for char {
    #[inline]
    fn size(&self) -> usize {
        std::mem::size_of::<char>()
    }

    #[inline]
    fn encode(&self) -> Vec<u8> {
        (*self as u32).encode()
    }

    #[inline]
    fn encode_to<W: Write>(&self, writer: W) -> Result<usize> {
        (*self as u32).encode_to(writer)
    }
}

impl<T> Encode for Option<T>
where
    T: Encode,
{
    fn size(&self) -> usize {
        match self {
            Some(value) => std::mem::size_of::<u8>() + value.size(),
            None => std::mem::size_of::<u8>(),
        }
    }

    fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.size());
        self.encode_to(&mut bytes)
            .expect("Unable to write into Vec");
        bytes
    }

    fn encode_to<W: Write>(&self, mut writer: W) -> Result<usize> {
        match self {
            None => writer.write(&[0]),
            Some(value) => Ok(writer.write(&[1])? + value.encode_to(writer)?),
        }
    }
}

impl<T> Encode for Vec<T>
where
    T: Encode,
{
    fn size(&self) -> usize {
        std::mem::size_of::<u64>() + self.iter().map(Encode::size).sum::<usize>()
    }

    fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.size());
        self.encode_to(&mut bytes)
            .expect("Unable to write into Vec");
        bytes
    }

    fn encode_to<W: Write>(&self, mut writer: W) -> Result<usize> {
        let mut encoded = 0;

        encoded += (self.len() as u64).encode_to(&mut writer)?;

        for item in self.iter() {
            encoded += item.encode_to(&mut writer)?;
        }

        Ok(encoded)
    }
}

impl<T> Encode for [T]
where
    T: Encode,
{
    fn size(&self) -> usize {
        std::mem::size_of::<u64>() + self.iter().map(Encode::size).sum::<usize>()
    }

    fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.size());
        self.encode_to(&mut bytes)
            .expect("Unable to write into Vec");
        bytes
    }

    fn encode_to<W: Write>(&self, mut writer: W) -> Result<usize> {
        let mut encoded = 0;

        encoded += (self.len() as u64).encode_to(&mut writer)?;

        for item in self.iter() {
            encoded += item.encode_to(&mut writer)?;
        }

        Ok(encoded)
    }
}
