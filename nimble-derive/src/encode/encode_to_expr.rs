use proc_macro2::TokenStream;

use crate::context::Context;

pub trait EncodeToExpr {
    fn encode_to_expr(&self) -> TokenStream;
}

impl<'a> EncodeToExpr for Context<'a> {
    fn encode_to_expr(&self) -> TokenStream {
        unimplemented!()
    }
}
