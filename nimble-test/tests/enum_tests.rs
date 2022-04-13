use futures_executor as executor;

use nimble::{decode, encode, Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
enum MyEnum {
    Unit,
    Unnamed(u8, u16),
    Named { a: u8, b: u16 },
}

#[test]
fn enum_unit_variant_test() {
    executor::block_on(async {
        let original = MyEnum::Unit;

        assert_eq!(1, original.size());
        let encoded = encode(&original).await;
        assert_eq!(encoded.len(), original.size());
        let decoded: MyEnum = decode(&encoded).await.unwrap();

        assert_eq!(original, decoded);
    });
}

#[test]
fn enum_unnamed_variant_test() {
    executor::block_on(async {
        let original = MyEnum::Unnamed(10, 20);

        assert_eq!(4, original.size());
        let encoded = encode(&original).await;
        assert_eq!(encoded.len(), original.size());
        let decoded: MyEnum = decode(&encoded).await.unwrap();

        assert_eq!(original, decoded);
    });
}

#[test]
fn enum_named_variant_test() {
    executor::block_on(async {
        let original = MyEnum::Named { a: 10, b: 20 };

        assert_eq!(4, original.size());
        let encoded = encode(&original).await;
        assert_eq!(encoded.len(), original.size());
        let decoded: MyEnum = decode(&encoded).await.unwrap();

        assert_eq!(original, decoded);
    });
}
