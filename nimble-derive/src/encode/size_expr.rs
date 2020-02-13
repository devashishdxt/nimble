use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{punctuated::Iter, spanned::Spanned, Field, Index};

use crate::{
    context::{Context, ExprType},
    util::{get_variant_pattern_match_expr, FieldExt, FieldsExt, VariantExt},
};

pub trait SizeExpr {
    fn size_expr(&self) -> TokenStream;
}

impl<'a> SizeExpr for Context<'a> {
    fn size_expr(&self) -> TokenStream {
        let field_prefix = &self.field_prefix;

        match &self.expr_type {
            ExprType::Struct { fields } => {
                size_calculation_expr(fields.clone(), &field_prefix, None)
            }
            ExprType::Enum { variants } => {
                let match_exprs = variants.clone().map(|variant| -> TokenStream {
                    let span = variant.span();
                    let variant_name = variant.get_name();
                    let fields_type = variant.fields.get_type();
                    let fields = &variant.fields;
                    let pattern_matching =
                        get_variant_pattern_match_expr(fields.iter_fields(), fields_type, true);
                    let size_calculation = size_calculation_expr(
                        fields.iter_fields(),
                        &field_prefix,
                        Some(quote! {core::mem::size_of::<u32>()}),
                    );

                    quote_spanned! {span=>
                        Self:: #variant_name #pattern_matching => #size_calculation
                    }
                });

                quote! {
                    match self {
                        #(#match_exprs,)*
                    }
                }
            }
        }
    }
}

fn size_calculation_expr(
    fields: Iter<Field>,
    field_prefix: &TokenStream,
    base_size: Option<TokenStream>,
) -> TokenStream {
    let recurse = fields.enumerate().map(|(i, f)| {
        let field_name = f.get_name();

        match field_name {
            Some(field_name) => quote_spanned! {f.span()=>
                Encode::size(#field_prefix #field_name)
            },
            None => {
                let index = Index::from(i);
                quote_spanned! {f.span()=>
                    Encode::size(#field_prefix #index)
                }
            }
        }
    });

    let base_size = base_size.unwrap_or_else(|| quote! {0});

    quote! {
        #base_size #(+ #recurse)*
    }
}
