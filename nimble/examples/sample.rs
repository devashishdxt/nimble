use nimble::Encode;

#[derive(Encode)]
struct MyStruct<T> where T: Send + Sync {
    a: u8,
    b: u16,
    arr: Vec<T>,
}

fn main() {}
