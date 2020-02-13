use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Iter, Data, Field, Variant};

use crate::util::{DataEnumExt, FieldsExt};

pub struct Context<'a> {
    pub expr_type: ExprType<'a>,
    pub field_prefix: TokenStream,
}

pub enum ExprType<'a> {
    Struct { fields: Iter<'a, Field> },
    Enum { variants: Iter<'a, Variant> },
}

impl<'a> Context<'a> {
    #[inline]
    pub fn new(data: &'a mut Data) -> Self {
        match *data {
            Data::Struct(ref data) => {
                let fields = data.fields.iter_fields();

                let expr_type = ExprType::Struct { fields };
                let field_prefix = quote!(&self.);

                Context {
                    expr_type,
                    field_prefix,
                }
            }
            Data::Enum(ref mut data) => {
                data.name_unnamed();
                let variants = data.iter_variants();
                let expr_type = ExprType::Enum { variants };
                let field_prefix = quote!();

                Context {
                    expr_type,
                    field_prefix,
                }
            }
            Data::Union(_) => panic!("`nimble::Encode` is not supported on unions"),
        }
    }
}
