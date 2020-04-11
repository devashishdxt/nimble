use core::convert::TryFrom;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{punctuated::Iter, spanned::Spanned, Field, Index};

use crate::{
    context::{Context, ExprType},
    util::{get_variant_pattern_match_expr, FieldExt, FieldsExt, VariantExt},
};

pub trait SizeExpr {
    /// Returns expression that goes in `Encode::size()` method
    fn size_expr(&self) -> TokenStream;
}

impl<'a> SizeExpr for Context<'a> {
    fn size_expr(&self) -> TokenStream {
        let name = &self.name;
        let field_prefix = &self.field_prefix;

        match &self.expr_type {
            ExprType::Struct { ref fields, .. } => {
                size_calculation_expr(fields.clone(), &field_prefix, None)
            }
            ExprType::Enum { ref variants } => {
                let match_exprs = variants
                    .clone()
                    .enumerate()
                    .map(|(i, variant)| -> TokenStream {
                        let span = variant.span();
                        let variant_name = variant.get_name();
                        let fields_type = variant.fields.get_type();
                        let fields = &variant.fields;
                        let pattern_matching =
                            get_variant_pattern_match_expr(fields.iter_fields(), fields_type, true);

                        let index = u128::try_from(i).expect("Failed to convert usize to u128. Log an issue on nimble's GitHub repository with backtrace.");
                        let base_size_expr = quote! {
                            Encode::size(&nimble::VarInt::from( #index ))
                        };

                        let size_calculation = size_calculation_expr(
                            fields.iter_fields(),
                            &field_prefix,
                            Some(base_size_expr),
                        );

                        quote_spanned! {span=>
                            #name :: #variant_name #pattern_matching => #size_calculation
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

/// Returns expression to calculate size of all the fields
///
/// # Arguments
///
/// - `fields`: An iterator over all the fields
/// - `field_prefix`: Prefix to apply before accessing each field (for example, `&self.` is a field prefix for accessing struct fields)
/// - `base_size`: Base size expression, if any (this expression is added to size calculation expression)
///
/// # Example
///
/// For below struct:
///
/// ```rust,ignore
/// struct MyStruct {
///     a: u8,
///     b: u16,
/// }
/// ```
///
/// This function will return:
///
/// ```ignore
/// 0 + Encode::size(&self.a) + Encode::size(&self.b)
/// ```
///
/// assuming `field_prefix = &self.` and `base_size = None`.
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
