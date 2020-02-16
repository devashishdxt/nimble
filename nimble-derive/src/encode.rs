mod encode_to_expr;
mod size_expr;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, DeriveInput};

use self::{encode_to_expr::EncodeToExpr, size_expr::SizeExpr};
use crate::{context::Context, util::add_trait_bounds};

pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    let mut input = parse_macro_input!(input as DeriveInput);

    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;

    // Add a bound `T: Encode` to every type parameter T.
    let generics = add_trait_bounds(input.generics, parse_quote!(Encode));
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Create context for generating expressions
    let context = Context::new(&name, &mut input.data);

    // Generate an expression for calculating size of encoded byte array.
    let size: TokenStream = context.size_expr();

    // Generate expression for encoding value to byte array and writing it to writer.
    let encode_to = context.encode_to_expr();

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        // The generated impl.
        #[nimble::async_trait]
        impl #impl_generics Encode for #name #ty_generics #where_clause {
            fn size(&self) -> usize {
                #size
            }

            async fn encode_to<W>(&self, config: &nimble::Config, mut writer: W) -> nimble::Result<usize>
            where
                W: nimble::io::Write + Unpin + Send,
            {
                #encode_to
            }
        }
    };

    // Hand the output tokens back to the compiler
    proc_macro::TokenStream::from(expanded)
}
