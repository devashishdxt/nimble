#![forbid(unsafe_code)]
#![deny(missing_docs, unstable_features)]
//! # Nimble
//!
//! Async friendly, simple and fast binary encoding/decoding in Rust.
//!
//! ## Binary encoding scheme
//!
//! This crate uses a minimal binary encoding scheme. For example, consider the following `struct`:
//!
//! ```
//! struct MyStruct {
//!     a: u8,
//!     b: u16,
//! }
//! ```
//!
//! `encode()` will serialize this into `Vec` of size `3` (which is the sum of sizes of `u8` and `u16`).
//!
//! Similarly, for types which can have dynamic size (`Vec`, `String`, etc.), `encode()` prepends the size of encoded value
//! as `u64`.
//!
//! ## Usage
//!
//! Add `nimble` in your `Cargo.toml`'s `dependencies` section:
//!
//! ```toml
//! [dependencies]
//! nimble = { version = "0.2", features = ["derive"] }
//! ```
//!
//! Or, if you are in an environment based on `tokio`, use:
//!
//! ```toml
//! [dependencies]
//! nimble = { version = "0.2", default-features = false, features = ["derive", "tokio"] }
//! ```
//!
//! For encoding and decoding, any type must implement two traits provided by this crate, i.e., `Encode` and `Decode`. For
//! convenience, `nimble` provides `derive` macros (only when `"derive"` feature is enabled) to implement these traits.
//!
//! ```rust,ignore
//! use nimble::{Encode, Decode};
//!
//! #[derive(Encode, Decode)]
//! struct MyStruct {
//!     a: u8,
//!     b: u16,
//! }
//! ```
//!
//! Now you can use `encode()` and `decode()` functions to encode and decode values of `MyStruct`. In addition to this, you
//! can also use `MyStruct::encode_to()` function to encode values directly to a type implementing `AsyncWrite` and
//! `MyStruct::decode_from()` function to decode values directly from a type implementing `AsyncRead`.
//!
//! > Note: Most of the functions exposed by this crate are `async` functions and returns `Future` values. So, you'll need
//! an executor to drive the `Future` returned from these functions. `async-std` and `tokio` are two popular options.
//!
//! ### Features
//!
//! - `futures`: Select this feature when you want to implement `Encode` and `Decode` using `futures`'
//!   `AsyncRead`/`AsyncWrite` traits.
//!   - **Enabled** by default.
//! - `tokio`: Select this feature when you want to implement `Encode` and `Decode` using `tokio`'s `AsyncRead`/`AsyncWrite`
//!   traits.
//!   - **Disabled** by default.
//! - `derive`: Enables derive macros for implementing `Encode` and `Decode` traits.
//!   - **Disabled** by default.
//!
//! > Note: Features `futures` and `tokio` are mutually exclusive, i.e., only one of them can be enabled at a time.
//! > Compilation will fail if either both of them are enabled or none of them are enabled.
#[cfg(all(feature = "futures", feature = "tokio"))]
compile_error!("Features `futures` and `tokio` are mutually exclusive");

#[cfg(not(any(feature = "futures", feature = "tokio")))]
compile_error!("Either feature `futures` or `tokio` must be enabled for this crate");

mod config;
mod decode;
mod encode;
mod error;

pub mod io;

#[cfg(feature = "derive")]
pub use nimble_derive::{Decode, Encode};

/// Utility macro for implementing [`Encode`](trait.Encode.html) and [`Decode`](trait.Decode.html) traits.
pub use async_trait::async_trait;

pub use self::{
    config::{Config, Endianness},
    decode::Decode,
    encode::Encode,
    error::{Error, Result},
};

use self::io::{Read, Write};

const DEFAULT_CONFIG: Config = Config::new_default();

/// Returns default `Config`
pub fn config<'a>() -> &'a Config {
    &DEFAULT_CONFIG
}

/// Encodes a value in a `Vec` using default configuration
#[inline]
pub async fn encode<E: Encode + ?Sized>(value: &E) -> Vec<u8> {
    DEFAULT_CONFIG.encode(value).await
}

/// Writes encoded byte array to writer and returns the number of bytes written
#[inline]
pub async fn encode_to<E: Encode + ?Sized, W: Write + Unpin + Send>(
    value: &E,
    writer: W,
) -> Result<usize> {
    DEFAULT_CONFIG.encode_to(value, writer).await
}

/// Decodes a value from bytes using default configuration
#[inline]
pub async fn decode<D: Decode, T: AsRef<[u8]>>(bytes: T) -> Result<D> {
    DEFAULT_CONFIG.decode(bytes).await
}

/// Decodes values from reader
#[inline]
pub async fn decode_from<D: Decode, R: Read + Unpin + Send>(reader: R) -> Result<D> {
    DEFAULT_CONFIG.decode_from(reader).await
}

#[cfg(test)]
#[cfg(not(feature = "tokio"))]
mod tests {
    use std::{
        collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque},
        ffi::CString,
    };

    use futures_executor as executor;
    use rand::random;

    use crate::{decode, encode, Encode};

    macro_rules! primitive_test {
        ($type: ty, $name: ident) => {
            #[test]
            fn $name() {
                executor::block_on(async {
                    let original = random::<$type>();
                    let encoded = encode(&original).await;
                    assert_eq!(original.size(), encoded.len());
                    let decoded: $type = decode(&encoded).await.unwrap();
                    assert_eq!(original, decoded, "Invalid encoding/decoding");
                });
            }
        };
    }

    primitive_test!(u8, u8_test);
    primitive_test!(u16, u16_test);
    primitive_test!(u32, u32_test);
    primitive_test!(u64, u64_test);
    primitive_test!(u128, u128_test);

    primitive_test!(i8, i8_test);
    primitive_test!(i16, i16_test);
    primitive_test!(i32, i32_test);
    primitive_test!(i64, i64_test);
    primitive_test!(i128, i128_test);

    primitive_test!(usize, usize_test);
    primitive_test!(isize, isize_test);
    primitive_test!(bool, bool_test);
    primitive_test!(char, char_test);

    primitive_test!(f32, f32_test);
    primitive_test!(f64, f64_test);

    primitive_test!([u8; 32], u8_arr_test);
    primitive_test!([u16; 32], u16_arr_test);
    primitive_test!([u32; 32], u32_arr_test);
    primitive_test!([u64; 32], u64_arr_test);
    primitive_test!([u128; 32], u128_arr_test);

    primitive_test!([i8; 32], i8_arr_test);
    primitive_test!([i16; 32], i16_arr_test);
    primitive_test!([i32; 32], i32_arr_test);
    primitive_test!([i64; 32], i64_arr_test);
    primitive_test!([i128; 32], i128_arr_test);

    primitive_test!([usize; 32], usize_arr_test);
    primitive_test!([isize; 32], isize_arr_test);
    primitive_test!([bool; 32], bool_arr_test);
    primitive_test!([char; 32], char_arr_test);

    primitive_test!([f32; 32], f32_arr_test);
    primitive_test!([f64; 32], f64_arr_test);

    #[test]
    fn option_none_test() {
        executor::block_on(async {
            let original: Option<u8> = None;
            let encoded = encode(&original).await;
            assert_eq!(original.size(), encoded.len());
            let decoded: Option<u8> = decode(&encoded).await.unwrap();
            assert_eq!(original, decoded, "Invalid encoding/decoding");
        });
    }

    #[test]
    fn option_some_test() {
        executor::block_on(async {
            let original: Option<u8> = Some(random());
            let encoded = encode(&original).await;
            assert_eq!(original.size(), encoded.len());
            let decoded: Option<u8> = decode(&encoded).await.unwrap();
            assert_eq!(original, decoded, "Invalid encoding/decoding");
        });
    }

    #[test]
    fn result_ok_test() {
        executor::block_on(async {
            let original: Result<u8, u8> = Ok(random());
            let encoded = encode(&original).await;
            assert_eq!(original.size(), encoded.len());
            let decoded: Result<u8, u8> = decode(&encoded).await.unwrap();
            assert_eq!(original, decoded, "Invalid encoding/decoding");
        });
    }

    #[test]
    fn result_err_test() {
        executor::block_on(async {
            let original: Result<u8, u8> = Err(random());
            let encoded = encode(&original).await;
            assert_eq!(original.size(), encoded.len());
            let decoded: Result<u8, u8> = decode(&encoded).await.unwrap();
            assert_eq!(original, decoded, "Invalid encoding/decoding");
        });
    }

    #[test]
    fn fixed_arr_test() {
        executor::block_on(async {
            let original = [1i32, 2i32, 3i32];
            let encoded = encode(&original).await;
            assert_eq!(original.size(), encoded.len());
            let decoded: [i32; 3] = decode(&encoded).await.unwrap();
            assert_eq!(original, decoded, "Invalid encoding/decoding");
        });
    }

    #[test]
    fn vec_test() {
        executor::block_on(async {
            let original = vec![1, 2, 3];
            let encoded = encode(&original).await;
            assert_eq!(original.size(), encoded.len());
            let decoded: Vec<i32> = decode(&encoded).await.unwrap();
            assert_eq!(original, decoded, "Invalid encoding/decoding");
        });
    }

    #[test]
    fn slice_test() {
        executor::block_on(async {
            let original = [1i32, 2i32, 3i32];
            let encoded = encode(&original[..]).await;
            assert_eq!(original[..].size(), encoded.len());
            let decoded: Vec<i32> = decode(&encoded).await.unwrap();
            assert_eq!(original.to_vec(), decoded, "Invalid encoding/decoding");
        });
    }

    #[test]
    fn string_test() {
        executor::block_on(async {
            let original = "hello";
            let encoded = encode(original).await;
            assert_eq!(original.size(), encoded.len());
            let decoded: String = decode(&encoded).await.unwrap();
            assert_eq!(original.to_string(), decoded, "Invalid encoding/decoding");
        })
    }

    #[test]
    fn c_string_test() {
        executor::block_on(async {
            let original = CString::new("hello").unwrap();
            let encoded = encode(&original).await;
            assert_eq!(original.size(), encoded.len());
            let decoded: CString = decode(&encoded).await.unwrap();
            assert_eq!(original, decoded, "Invalid encoding/decoding");
        })
    }

    #[test]
    fn vec_string_test() {
        executor::block_on(async {
            let original = vec!["hello".to_string(), "world".to_string()];
            let encoded = encode(&original).await;
            assert_eq!(original.size(), encoded.len());
            let decoded: Vec<String> = decode(&encoded).await.unwrap();
            assert_eq!(original, decoded, "Invalid encoding/decoding");
        })
    }

    #[test]
    fn box_test() {
        executor::block_on(async {
            let original = Box::new("10".to_string());
            let encoded = encode(&original).await;
            assert_eq!(original.size(), encoded.len());
            let decoded: Box<String> = decode(&encoded).await.unwrap();
            assert_eq!(original, decoded, "Invalid encoding/decoding");
        });
    }

    #[test]
    fn tuple_test() {
        executor::block_on(async {
            let original = ("hello".to_string(), 25u8, 100i32);
            let encoded = encode(&original).await;
            assert_eq!(original.size(), encoded.len());
            let decoded: (String, u8, i32) = decode(&encoded).await.unwrap();
            assert_eq!(original, decoded, "Invalid encoding/decoding");
        });
    }

    #[test]
    fn vec_deque_test() {
        executor::block_on(async {
            let mut original = VecDeque::with_capacity(3);
            original.push_back(1);
            original.push_back(2);
            original.push_back(3);
            let encoded = encode(&original).await;
            assert_eq!(original.size(), encoded.len());
            let decoded: VecDeque<i32> = decode(&encoded).await.unwrap();
            assert_eq!(original, decoded, "Invalid encoding/decoding");
        });
    }

    #[test]
    fn linked_list_test() {
        executor::block_on(async {
            let mut original = LinkedList::new();
            original.push_back(1);
            original.push_back(2);
            original.push_back(3);
            let encoded = encode(&original).await;
            assert_eq!(original.size(), encoded.len());
            let decoded: LinkedList<i32> = decode(&encoded).await.unwrap();
            assert_eq!(original, decoded, "Invalid encoding/decoding");
        });
    }

    #[test]
    fn hash_set_test() {
        executor::block_on(async {
            let mut original = HashSet::with_capacity(3);
            original.insert(1);
            original.insert(2);
            original.insert(3);
            let encoded = encode(&original).await;
            assert_eq!(original.size(), encoded.len());
            let decoded: HashSet<i32> = decode(&encoded).await.unwrap();
            assert_eq!(original, decoded, "Invalid encoding/decoding");
        });
    }

    #[test]
    fn btree_set_test() {
        executor::block_on(async {
            let mut original = BTreeSet::new();
            original.insert(3);
            original.insert(1);
            original.insert(2);
            let encoded = encode(&original).await;
            assert_eq!(original.size(), encoded.len());
            let decoded: BTreeSet<i32> = decode(&encoded).await.unwrap();
            assert_eq!(original, decoded, "Invalid encoding/decoding");
        });
    }

    #[test]
    fn binary_heap_test() {
        executor::block_on(async {
            let mut original: BinaryHeap<i32> = BinaryHeap::new();
            original.push(3);
            original.push(1);
            original.push(2);
            let encoded = encode(&original).await;
            assert_eq!(original.size(), encoded.len());
            let decoded: BinaryHeap<i32> = decode(&encoded).await.unwrap();
            let new_encoded = encode(&decoded).await;
            assert_eq!(encoded, new_encoded, "Invalid encoding/decoding");
        });
    }

    #[test]
    fn hash_map_test() {
        executor::block_on(async {
            let mut original = HashMap::with_capacity(3);
            original.insert(1, "Hello".to_owned());
            original.insert(2, "World".to_owned());
            original.insert(3, "!".to_owned());
            let encoded = encode(&original).await;
            assert_eq!(original.size(), encoded.len());
            let decoded: HashMap<i32, String> = decode(&encoded).await.unwrap();
            assert_eq!(original, decoded, "Invalid encoding/decoding");
        });
    }

    #[test]
    fn btree_map_test() {
        executor::block_on(async {
            let mut original = BTreeMap::new();
            original.insert(1, "Hello".to_owned());
            original.insert(2, "World".to_owned());
            original.insert(3, "!".to_owned());
            let encoded = encode(&original).await;
            assert_eq!(original.size(), encoded.len());
            let decoded: BTreeMap<i32, String> = decode(&encoded).await.unwrap();
            assert_eq!(original, decoded, "Invalid encoding/decoding");
        });
    }
}
