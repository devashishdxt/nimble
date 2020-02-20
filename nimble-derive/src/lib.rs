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
//! nimble = { version = "0.2", features = ["derive", "tokio"] }
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
//! - `tokio`: Select this feature when you are using `tokio`'s executor to drive `Future` values returned by functions in
//!   this crate. This implements `Encode` and `Decode` using `tokio`'s `AsyncRead`/`AsyncWrite` traits.
//!   - **Disabled** by default.
//! - `derive`: Enables derive macros for implementing `Encode` and `Decode` traits.
//!   - **Disabled** by default.
extern crate proc_macro;

mod context;
mod decode;
mod encode;
mod util;

#[proc_macro_derive(Encode)]
/// Derive macro to implement `Encode` trait
pub fn derive_encode(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    encode::derive(input)
}

#[proc_macro_derive(Decode)]
/// Derive macro to implement `Decode` trait
pub fn derive_decode(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    decode::derive(input)
}
