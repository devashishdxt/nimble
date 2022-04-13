use futures_executor as executor;

use nimble::{decode, encode, Decode, Encode};

#[test]
fn unit_struct_test() {
    #[derive(Debug, PartialEq, Encode, Decode)]
    struct UnitStruct;

    executor::block_on(async {
        let original = UnitStruct;

        assert_eq!(0, original.size());
        let encoded = encode(&original).await;
        assert_eq!(encoded.len(), original.size());
        let decoded: UnitStruct = decode(&encoded).await.unwrap();

        assert_eq!(original, decoded);
    });
}

#[test]
fn unnamed_struct_test() {
    #[derive(Debug, PartialEq, Encode, Decode)]
    struct UnnamedStruct(u8, u16);

    executor::block_on(async {
        let original = UnnamedStruct(10, 20);

        assert_eq!(3, original.size());
        let encoded = encode(&original).await;
        assert_eq!(encoded.len(), original.size());
        let decoded: UnnamedStruct = decode(&encoded).await.unwrap();

        assert_eq!(original, decoded);
    });
}

#[test]
fn named_struct_test() {
    #[derive(Debug, PartialEq, Encode, Decode)]
    struct NamedStruct {
        a: u8,
        b: u16,
    }

    executor::block_on(async {
        let original = NamedStruct { a: 10, b: 20 };

        assert_eq!(3, original.size());
        let encoded = encode(&original).await;
        assert_eq!(encoded.len(), original.size());
        let decoded: NamedStruct = decode(&encoded).await.unwrap();

        assert_eq!(original, decoded);
    });
}
