use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::enums::*;
use crate::inputs::*;
use crate::structs::*;
use crate::utils::*;

pub struct Definition {
    pub ty: syn::Type,
    pub definition: Option<TokenStream>,
    // For `Desc` types, the method body is for `describe`.
    // For `Change` types, the method body is for `comparison`.
    pub method_body: TokenStream,
}

impl Definition {
    pub fn ident_to_type(ident: &syn::Ident) -> syn::Type {
        syn::parse2(quote!(#ident)).unwrap_or_else(|_| panic!("Failed to parse type"))
    }

    pub fn assoc_type(ty: &syn::Type, name: &str) -> syn::Type {
        let ident = format_ident!("{}", name);
        syn::parse2(quote!(<#ty as comparable::Comparable>::#ident))
            .unwrap_or_else(|_| panic!("Failed to parse associated type"))
    }

    pub fn changed_type(ty: &syn::Type) -> syn::Type {
        syn::parse2(quote!(comparable::Changed<#ty>))
            .unwrap_or_else(|_| panic!("Failed to parse Changed type"))
    }

    //
    // Desc
    //
    pub fn generate_desc_from_data(inputs: &Inputs) -> Self {
        let type_name = &inputs.input.ident;
        let desc_name = format_ident!("{}Desc", &inputs.input.ident);
        let desc_type = generate_type_definition(
            &inputs.visibility,
            &desc_name,
            &map_on_fields_over_data(&inputs.input.data, |_, field| syn::Field {
                ty: Self::assoc_type(&field.ty, "Desc"),
                ..field.clone()
            }),
        );
        Self {
            ty: inputs
                .attrs
                .describe_type
                .as_ref()
                .unwrap_or(
                    &syn::parse2(if inputs.attrs.compare_default {
                        quote!(Self::Change)
                    } else {
                        quote!(#desc_name)
                    })
                    .expect("Failed to parse Desc type name"),
                )
                .clone(),
            definition: if inputs.attrs.describe_type.is_some() || inputs.attrs.compare_default {
                None
            } else {
                Some(quote!(#desc_type))
            },
            method_body: inputs
                .attrs
                .describe_body
                .as_ref()
                .map(
                    #[allow(unused_variables)] // compiler doesn't see the use of x
                    |x| quote!(#x),
                )
                .unwrap_or(if inputs.attrs.compare_default {
                    quote!(#type_name::default().comparison(self).unwrap_or_default())
                } else {
                    Self::generate_describe_body(
                        &inputs.input.ident,
                        &desc_name,
                        &inputs.input.data,
                    )
                }),
        }
    }

    //
    // describe
    //
    fn generate_describe_body(
        type_name: &syn::Ident,
        desc_name: &syn::Ident,
        data: &syn::Data,
    ) -> TokenStream {
        match data {
            syn::Data::Struct(st) => generate_describe_body_for_structs(desc_name, st),
            syn::Data::Enum(en) => generate_describe_body_for_enums(type_name, desc_name, en),
            syn::Data::Union(_un) => {
                panic!("comparable_derive::generate_describe_body not implemented for unions")
            }
        }
    }

    //
    // Change
    //
    pub fn generate_change_from_data(inputs: &Inputs) -> Self {
        let type_name = &inputs.input.ident;
        let change_name = format_ident!("{}Change", type_name);
        let change_type = generate_type_definition(
            &inputs.visibility,
            &change_name,
            &Self::create_change_type(&inputs.input.ident, &inputs.input.data),
        );
        Self {
            ty: syn::parse2(if is_struct_with_many_fields(&inputs.input.data) {
                quote!(Vec<#change_name>)
            } else {
                quote!(#change_name)
            })
            .expect("Failed to parse Change type name"),
            definition: Some(quote!(#change_type)),
            method_body: Self::generate_comparison_body(
                type_name,
                &change_name,
                &inputs.input.data,
            ),
        }
    }

    fn create_change_type(type_name: &syn::Ident, data: &syn::Data) -> syn::Data {
        match data {
            syn::Data::Struct(st) => create_change_type_for_structs(st),
            syn::Data::Enum(en) => create_change_type_for_enums(type_name, en),
            syn::Data::Union(_un) => {
                panic!("comparable_derive::generate_change_type not implemented for unions")
            }
        }
    }

    //
    // comparison
    //
    fn generate_comparison_body(
        type_name: &syn::Ident,
        change_name: &syn::Ident,
        data: &syn::Data,
    ) -> TokenStream {
        match data {
            syn::Data::Struct(st) => generate_comparison_body_for_structs(change_name, st),
            syn::Data::Enum(en) => {
                if en.variants.is_empty() {
                    quote! {
                        comparable::Changed::Unchanged
                    }
                } else {
                    EnumDetails::from(type_name, change_name, en)
                        .generate_comparison_body(change_name)
                }
            }
            syn::Data::Union(_un) => {
                panic!("comparable_derive::generate_comparison_body not implemented for unions")
            }
        }
    }
}

impl quote::ToTokens for Definition {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(body) = &self.definition {
            *tokens = quote!(#body);
        }
    }
}
