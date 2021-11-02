use proc_macro2::Span;

use crate::attrs::*;
use crate::definition::*;
use crate::outputs::*;

pub struct Inputs<'a> {
    pub attrs: Attributes,
    pub input: &'a syn::DeriveInput,
    pub visibility: syn::Visibility,
}

impl<'a> Inputs<'a> {
    pub fn from(input: &'a syn::DeriveInput) -> Self {
        let attrs = Attributes::from(&input.attrs);

        let visibility = if attrs.comparable_private {
            syn::Visibility::Inherited
        } else if attrs.comparable_public {
            syn::Visibility::Public(syn::VisPublic {
                pub_token: syn::token::Pub {
                    span: Span::call_site(),
                },
            })
        } else {
            input.vis.clone()
        };

        Inputs {
            attrs,
            input,
            visibility,
        }
    }

    pub fn process_data(&self) -> Outputs {
        let is_unitary = match &self.input.data {
            syn::Data::Struct(st) => match &st.fields {
                syn::Fields::Unit => true,
                syn::Fields::Unnamed(unnamed) => unnamed.unnamed.is_empty(),
                syn::Fields::Named(named) => named.named.is_empty(),
            },
            syn::Data::Enum(en) => en.variants.is_empty(),
            syn::Data::Union(_st) => {
                panic!("Comparable derivation not available for unions");
            }
        };
        self.process_struct_or_enum(is_unitary)
    }

    fn process_struct_or_enum(&self, is_unitary: bool) -> Outputs {
        Outputs {
            desc: if self.attrs.no_description {
                None
            } else {
                Some(Definition::generate_desc_type(self))
            },
            change: if is_unitary {
                None
            } else {
                Some(Definition::generate_change_type(self))
            },
        }
    }
}
