mod decode;
mod encode;
mod error;

pub use decode::Decode;
pub use encode::Encode;
pub use error::{Error, Result};

pub async fn encode<E: Encode>(value: E) -> Result<Vec<u8>> {
    let mut bytes = Vec::with_capacity(value.size());
    value.encode_to(&mut bytes).await?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use crate::encode;

    #[tokio::test]
    async fn check_primitive_encode() {
        let a = 1u8;
        assert_eq!(vec![1], encode(a).await.unwrap());

        let a = 2u16;
        assert_eq!(vec![0, 2], encode(a).await.unwrap());
    }
}
