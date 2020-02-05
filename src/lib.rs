mod decode;
mod encode;
mod error;

pub use decode::Decode;
pub use encode::Encode;
pub use error::{Error, Result};

pub async fn encode<E: Encode + ?Sized>(value: &E) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(value.size());
    // This will never fail because `encode_to()` returns `Err` only then there is an IO error which cannot happen when
    // writing to a `Vec`
    let _ = value.encode_to(&mut bytes).await.expect(
        "Failed to encode value. Log an issue on nimble's GitHub repository with backtrace.",
    );
    bytes
}

#[inline]
pub async fn decode<D: Decode>(bytes: &[u8]) -> Result<D> {
    D::decode_from(&mut bytes.as_ref()).await
}

#[cfg(test)]
mod tests {
    use rand::random;

    use crate::{decode, encode, Encode};

    macro_rules! primitive_test {
        ($type: ty, $name: ident) => {
            #[tokio::test]
            async fn $name() {
                let original = random::<$type>();
                let encoded = encode(&original).await;
                assert_eq!(original.size(), encoded.len());
                let decoded = decode::<$type>(&encoded).await.unwrap();
                assert_eq!(original, decoded, "Invalid encoding/decoding");
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

    #[tokio::test]
    async fn option_none_test() {
        let original: Option<u8> = None;
        let encoded = encode(&original).await;
        assert_eq!(original.size(), encoded.len());
        let decoded = decode::<Option<u8>>(&encoded).await.unwrap();
        assert_eq!(original, decoded, "Invalid encoding/decoding");
    }

    #[tokio::test]
    async fn option_some_test() {
        let original: Option<u8> = Some(random());
        let encoded = encode(&original).await;
        assert_eq!(original.size(), encoded.len());
        let decoded = decode::<Option<u8>>(&encoded).await.unwrap();
        assert_eq!(original, decoded, "Invalid encoding/decoding");
    }

    #[tokio::test]
    async fn fixed_arr_test() {
        let original = [1i32, 2i32, 3i32];
        let encoded = encode(&original).await;
        assert_eq!(original.size(), encoded.len());
        let decoded = decode::<[i32; 3]>(&encoded).await.unwrap();
        assert_eq!(original, decoded, "Invalid encoding/decoding");
    }

    #[tokio::test]
    async fn vec_test() {
        let original = vec![1, 2, 3];
        let encoded = encode(&original).await;
        assert_eq!(original.size(), encoded.len());
        let decoded = decode::<Vec<i32>>(&encoded).await.unwrap();
        assert_eq!(original, decoded, "Invalid encoding/decoding");
    }

    #[tokio::test]
    async fn slice_test() {
        let original = [1i32, 2i32, 3i32];
        let encoded = encode(&original[..]).await;
        assert_eq!(original[..].size(), encoded.len());
        let decoded = decode::<Vec<i32>>(&encoded).await.unwrap();
        assert_eq!(original.to_vec(), decoded, "Invalid encoding/decoding");
    }

    #[tokio::test]
    async fn string_test() {
        let original = "hello";
        let encoded = encode(original).await;
        assert_eq!(original.size(), encoded.len());
        let decoded = decode::<String>(&encoded).await.unwrap();
        assert_eq!(original.to_string(), decoded, "Invalid encoding/decoding");
    }

    #[tokio::test]
    async fn vec_string_test() {
        let original = vec!["hello".to_string(), "world".to_string()];
        let encoded = encode(&original).await;
        assert_eq!(original.size(), encoded.len());
        let decoded = decode::<Vec<String>>(&encoded).await.unwrap();
        assert_eq!(original, decoded, "Invalid encoding/decoding");
    }
}
