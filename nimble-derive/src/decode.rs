use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Fields, Ident};

use crate::util::add_trait_bounds;

pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;

    // Add a bound `T: Decode` to every type parameter T.
    let generics = add_trait_bounds(input.generics, parse_quote!(Decode));
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Generate expression for decoding value from reader.
    let decode_from = decode_from_impl(&name, &input.data);

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

// Generate expression for decoding value from reader.
fn decode_from_impl(name: &Ident, data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    // Expands to an expression like
                    //
                    //     MyStruct {
                    //         a: A::decode_from(&mut reader).await?,
                    //         b: B::decode_from(&mut reader).await?,
                    //     }
                    //
                    // but using fully qualified function call syntax.
                    //
                    // We take some care to use the span of each `syn::Field` as the span of the corresponding `size`
                    // call. This way if one of the field types does not implement `Encode` then the compiler's error
                    // message underlines which field it is.
                    let recurse = fields.named.iter().map(|f| {
                        let field_name = &f.ident;
                        let field_type = &f.ty;
                        quote_spanned! {f.span()=>
                            #field_name: <#field_type>::decode_from(&mut reader).await?
                        }
                    });
                    quote! {
                        Ok(#name {
                            #(#recurse,)*
                        })
                    }
                }
                Fields::Unnamed(ref fields) => {
                    // Expands to an expression like
                    //
                    //     MyStruct(
                    //         A::decode_from(&mut reader).await?,
                    //         B::decode_from(&mut reader).await?,
                    //     )
                    let recurse = fields.unnamed.iter().map(|f| {
                        let field_type = &f.ty;
                        quote_spanned! {f.span()=>
                            <#field_type>::decode_from(&mut reader).await?
                        }
                    });
                    quote! {
                        Ok(#name( #(#recurse,)*))
                    }
                }
                Fields::Unit => {
                    // Unit structs always generate encoded byte array of size zero.
                    quote!(Ok(#name))
                }
            }
        }
        Data::Enum(_) => unimplemented!(),
        Data::Union(_) => unimplemented!(),
    }
}
