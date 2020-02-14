mod decode_from_expr;

use quote::quote;
use syn::{parse_macro_input, parse_quote, DeriveInput};

use self::decode_from_expr::DecodeFromExpr;
use crate::{context::Context, util::add_trait_bounds};

pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    let mut input = parse_macro_input!(input as DeriveInput);

    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;

    // Add a bound `T: Decode` to every type parameter T.
    let generics = add_trait_bounds(input.generics, parse_quote!(Decode));
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Create context for generating expressions
    let context = Context::new(&name, &mut input.data);

    // Generate expression for decoding value from reader.
    let decode_from = context.decode_from_expr();

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        // The generated impl.
        #[nimble::async_trait]
        impl #impl_generics Decode for #name #ty_generics #where_clause {
            async fn decode_from<R>(mut reader: R) -> nimble::Result<Self>
            where
                R: nimble::io::Read + Unpin + Send,
            {
                #decode_from
            }
        }
    };

    // Hand the output tokens back to the compiler
    proc_macro::TokenStream::from(expanded)
}
