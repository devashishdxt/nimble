use crate::{Decode, Encode, Result};

/// Encoding/decoding configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Denotes endianness of encoded bytes
    pub endianness: Endianness,
}

impl Config {
    #[inline]
    /// Returns default configuration
    pub const fn new_default() -> Self {
        Self {
            endianness: Endianness::new_default(),
        }
    }

    /// Encodes a value in a `Vec`
    pub async fn encode<E: Encode + ?Sized>(&self, value: &E) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(value.size());
        // This will never fail because `encode_to()` returns `Err` only then there is an IO error which cannot happen when
        // writing to a `Vec`
        let _ = value.encode_to(self, &mut bytes).await.expect(
            "Failed to encode value. Log an issue on nimble's GitHub repository with backtrace.",
        );
        bytes
    }

    #[inline]
    /// Decodes a value from bytes
    pub async fn decode<D: Decode, T: AsRef<[u8]>>(&self, bytes: T) -> Result<D> {
        D::decode_from(self, &mut bytes.as_ref()).await
    }
}

impl Default for Config {
    #[inline]
    fn default() -> Self {
        Self::new_default()
    }
}

/// Endianness of encoded bytes
#[derive(Debug, Clone, Copy)]
pub enum Endianness {
    /// Little endian order
    LittleEndian,
    /// Big endian order
    BigEndian,
}

impl Endianness {
    #[inline]
    /// Returns default endianness
    pub const fn new_default() -> Self {
        Self::LittleEndian
    }
}

impl Default for Endianness {
    #[inline]
    fn default() -> Self {
        Self::new_default()
    }
}
