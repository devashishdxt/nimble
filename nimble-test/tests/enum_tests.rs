use nimble::{decode, encode, Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
enum MyEnum {
    UnitVariant,
    UnnamedVariant(u8, u16),
    NamedVariant { a: u8, b: u16 },
}

#[tokio::test]
async fn enum_unit_variant_test() {
    let original = MyEnum::UnitVariant;

    assert_eq!(4, original.size());
    let encoded = encode(&original).await;
    assert_eq!(encoded.len(), original.size());
    let decoded: MyEnum = decode(&encoded).await.unwrap();

    assert_eq!(original, decoded);
}

#[tokio::test]
async fn enum_unnamed_variant_test() {
    let original = MyEnum::UnnamedVariant(10, 20);

    assert_eq!(7, original.size());
    let encoded = encode(&original).await;
    assert_eq!(encoded.len(), original.size());
    let decoded: MyEnum = decode(&encoded).await.unwrap();

    assert_eq!(original, decoded);
}

#[tokio::test]
async fn enum_named_variant_test() {
    let original = MyEnum::NamedVariant { a: 10, b: 20 };

    assert_eq!(7, original.size());
    let encoded = encode(&original).await;
    assert_eq!(encoded.len(), original.size());
    let decoded: MyEnum = decode(&encoded).await.unwrap();

    assert_eq!(original, decoded);
}
