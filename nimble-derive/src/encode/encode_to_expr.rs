use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{punctuated::Iter, spanned::Spanned, Field, Index};

use crate::{
    context::{Context, ExprType},
    util::{get_variant_pattern_match_expr, FieldExt, FieldsExt, VariantExt},
};

pub trait EncodeToExpr {
    fn encode_to_expr(&self) -> TokenStream;
}

impl<'a> EncodeToExpr for Context<'a> {
    fn encode_to_expr(&self) -> TokenStream {
        let name = &self.name;
        let field_prefix = &self.field_prefix;

        match &self.expr_type {
            ExprType::Struct { ref fields, .. } => {
                bytes_encoding_expr(fields.clone(), field_prefix, None)
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
                        let variant_index = i as u128;
                        let bytes_encoding = bytes_encoding_expr(
                            fields.iter_fields(),
                            field_prefix,
                            Some(quote! {Encode::encode_to(& nimble::VarInt::from( #variant_index ), config, &mut writer).await?}),
                        );

                        quote_spanned! {span=>
                            #name :: #variant_name #pattern_matching => #bytes_encoding
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

/// Returns expression to encode all the fields
///
/// # Arguments
///
/// - `fields`: An iterator over all the fields
/// - `field_prefix`: Prefix to apply before accessing each field (for example, `&self.` is a field prefix for accessing struct fields)
/// - `base_expr`: Base encoding expression, if any (this expression is added to encoding expression)
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
/// Ok(0 + Encode::encode_to(&self.a, config, &mut writer).await? + Encode::encode_to(&self.b, config, &mut writer).await?)
/// ```
///
/// assuming `field_prefix = &self.` and `base_expr = None`.
fn bytes_encoding_expr(
    fields: Iter<'_, Field>,
    field_prefix: &TokenStream,
    base_expr: Option<TokenStream>,
) -> TokenStream {
    let recurse = fields.enumerate().map(|(i, f)| {
        let field_name = f.get_name();

        match field_name {
            Some(field_name) => quote_spanned! {f.span()=>
                Encode::encode_to(#field_prefix #field_name, config, &mut writer).await?
            },
            None => {
                let index = Index::from(i);
                quote_spanned! {f.span()=>
                    Encode::encode_to(#field_prefix #index, config, &mut writer).await?
                }
            }
        }
    });

    let base_expr = base_expr.unwrap_or_else(|| quote! {0});

    quote! {
        Ok(#base_expr #(+ #recurse)*)
    }
}
