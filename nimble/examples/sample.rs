use nimble::Encode;

#[derive(Encode)]
enum MyEnum {
    Hello(u8, u16),
    World { a: u8, b: u16 },
}

#[derive(Encode)]
struct MyStruct(u8, u16);

#[derive(Encode)]
struct MyNewStruct {
    a: u8,
    b: u16,
}

fn main() {}
