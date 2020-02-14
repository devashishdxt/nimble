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
