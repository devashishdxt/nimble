# Nimble

Async friendly, simple and fast binary encoding/decoding in Rust.

## Binary encoding scheme

This crate uses a minimal binary encoding scheme. For example, consider the following `struct`:

```
struct MyStruct {
    a: u8,
    b: u16,
}
```

`nimble::encode()` will serialize this into `Vec` of size `3` (which is the sum of sizes of `u8` and `u16`).

Similarly, for types which can have dynamic size (`Vec`, `String`, etc.), `nimble::encode()` prepends the size of
encoded value as `u64`.

> Note: `nimble`, by default, uses [big endian](https://en.wikipedia.org/wiki/Endianness#Big-endian) order to encode
values.

## Usage

Add `nimble` in your `Cargo.toml`'s `dependencies` section:

```toml
[dependencies]
nimble = "0.1"
```

For encoding and decoding, any type must implement two traits provided by this crate, i.e., `Encode` and `Decode`. For
convenience, `nimble` provides `derive` macros to implement these traits.

```rust
use nimble::{Encode, Decode};

#[derive(Encode, Decode)]
struct MyStruct {
    a: u8,
    b: u16,
}
```

Now you can use `nimble::encode()` and `nimble::decode()` functions to encode and decode values of `MyStruct`. In
addition to this, you can also use `MyStruct::encode_to()` function to encode values directly to a type implementing
`AsyncWrite` and `MyStruct::decode_from()` function to decode values directly from a type implementing `AsyncRead`.

> Note: Most of the functions exposed by this crate are `async` functions and returns `Future` values. So, you'll need
an executor to drive the `Future` returned from these functions. `async-std` and `tokio` are two popular options.

### Features

- `tokio`: Select this feature when you are using `tokio`'s executor to drive `Future` values returned by functions in
  this crate.
  - **Enabled** by default.
- `async-std`: Select this feature when you are using `async-std`'s executor to drive `Future` values returned by
  functions in this crate.
  - **Disabled** by default.

## License

Licensed under either of

- Apache License, Version 2.0 (http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as 
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
