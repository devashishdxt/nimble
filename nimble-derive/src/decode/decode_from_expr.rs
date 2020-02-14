use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{punctuated::Iter, spanned::Spanned, Field};

use crate::{
    context::{Context, ExprType},
    util::{FieldExt, FieldsExt, FieldsType, VariantExt},
};

pub trait DecodeFromExpr {
    fn decode_from_expr(&self) -> TokenStream;
}

impl<'a> DecodeFromExpr for Context<'a> {
    fn decode_from_expr(&self) -> TokenStream {
        let name = &self.name;

        match &self.expr_type {
            ExprType::Struct {
                ref fields_type,
                ref fields,
            } => decode_bytes_expr(name, *fields_type, fields.clone()),
            ExprType::Enum { ref variants } => {
                let match_exprs = variants
                    .clone()
                    .enumerate()
                    .map(|(i, variant)| -> TokenStream {
                        let variant_name = variant.get_name();
                        let fields_type = variant.fields.get_type();
                        let fields = variant.fields.iter_fields();

                        let decode_bytes_expr =
                            decode_bytes_expr(&quote!(#name :: #variant_name), fields_type, fields);
                        let index = i as u32;

                        quote_spanned! {variant.span()=>
                            #index => #decode_bytes_expr
                        }
                    });

                quote! {
                    let option = <u32>::decode_from(&mut reader).await?;

                    match option {
                        #(#match_exprs,)*
                        _ => Err(nimble::Error::InvalidEnumVariant(option)),
                    }
                }
            }
        }
    }
}

fn decode_bytes_expr<T: ToTokens>(
    name: &T,
    fields_type: FieldsType,
    fields: Iter<Field>,
) -> TokenStream {
    let field_exprs = fields.map(|f| -> TokenStream {
        let field_type = &f.get_type();

        match fields_type {
            FieldsType::Named => {
                let field_name = &f
                    .get_name()
                    .expect("Named fields are expected to have identifiers");

                quote_spanned! {f.span()=>
                    #field_name: <#field_type>::decode_from(&mut reader).await?
                }
            }
            FieldsType::Unnamed => {
                quote_spanned! {f.span()=>
                    <#field_type>::decode_from(&mut reader).await?
                }
            }
            FieldsType::Unit => {
                panic!("Unit structs or enum variants are not expected to have fields")
            }
        }
    });

    match fields_type {
        FieldsType::Named => {
            quote! {
                Ok(#name {
                    #(#field_exprs,)*
                })
            }
        }
        FieldsType::Unnamed => {
            quote! {
                Ok(#name (
                    #(#field_exprs,)*
                ))
            }
        }
        FieldsType::Unit => quote!(Ok(#name)),
    }
}
