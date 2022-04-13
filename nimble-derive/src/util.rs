use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    punctuated::Iter, spanned::Spanned, DataEnum, Field, Fields, GenericParam, Generics, Ident,
    Type, TypeParamBound, Variant,
};

pub trait FieldExt {
    /// Returns name of the field
    fn get_name(&self) -> Option<&Ident>;

    /// Returns type of the field
    fn get_type(&self) -> &Type;
}

impl FieldExt for Field {
    #[inline]
    fn get_name(&self) -> Option<&Ident> {
        self.ident.as_ref()
    }

    #[inline]
    fn get_type(&self) -> &Type {
        &self.ty
    }
}

/// Type of fields in a struct or enum variant
#[derive(Debug, Clone, Copy)]
pub enum FieldsType {
    /// Named fields { a: u8, b: u16 }
    Named,
    /// Unnamed fields (u8, u16)
    Unnamed,
    /// ()
    Unit,
}

pub trait FieldsExt {
    /// Get type of fields. (Named, Unnamed or Unit)
    fn get_type(&self) -> FieldsType;

    /// Get an iterator over all the fields.
    fn iter_fields(&self) -> Iter<'_, Field>;
}

impl FieldsExt for Fields {
    fn get_type(&self) -> FieldsType {
        match self {
            Fields::Named(_) => FieldsType::Named,
            Fields::Unnamed(_) => FieldsType::Unnamed,
            Fields::Unit => FieldsType::Unit,
        }
    }

    #[inline]
    fn iter_fields(&self) -> Iter<'_, Field> {
        self.iter()
    }
}

pub trait VariantExt {
    /// Returns name of enum variant
    fn get_name(&self) -> &Ident;
}

impl VariantExt for Variant {
    #[inline]
    fn get_name(&self) -> &Ident {
        &self.ident
    }
}

pub trait DataEnumExt {
    /// Returns all the variants in an enum
    fn iter_variants(&self) -> Iter<'_, Variant>;

    /// Names all the unnamed fields (default naming is `field_0, field_1, ...`)
    fn name_unnamed(&mut self);
}

impl DataEnumExt for DataEnum {
    #[inline]
    fn iter_variants(&self) -> Iter<'_, Variant> {
        self.variants.iter()
    }

    fn name_unnamed(&mut self) {
        self.variants.iter_mut().for_each(|variant| {
            variant
                .fields
                .iter_mut()
                .enumerate()
                .for_each(|(i, field)| {
                    let span = field.span();
                    if field.ident.is_none() {
                        field.ident = Some(Ident::new(&format!("field_{}", i), span));
                    }
                })
        });
    }
}

/// Add a bound `T: <bound>` to every type parameter T.
pub fn add_trait_bounds(mut generics: Generics, bound: TypeParamBound) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(bound.clone());
        }
    }
    generics
}

/// Returns expression of field names used for pattern matching.
///
/// # Example
///
/// 1. For named fields: { ref a, ref b, ref c }
/// 2. For unnamed fields: (ref field_0, ref field_1, ref field_2)
pub fn get_variant_pattern_match_expr(
    fields: Iter<'_, Field>,
    fields_type: FieldsType,
    add_ref: bool,
) -> TokenStream {
    let fields = fields.map(|f| {
        let field_name = f
            .get_name()
            .expect("Fields should have a name when writing pattern matching expression");
        if add_ref {
            quote_spanned! {f.span()=>
                ref #field_name
            }
        } else {
            quote_spanned! {f.span()=>
                #field_name
            }
        }
    });

    match fields_type {
        FieldsType::Named => {
            quote! {
                {
                    #(#fields,)*
                }
            }
        }
        FieldsType::Unnamed => {
            quote! {
                (
                    #(#fields,)*
                )
            }
        }
        FieldsType::Unit => quote!(),
    }
}
