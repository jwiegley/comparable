use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};

use crate::attrs::*;
use crate::enums::*;
use crate::inputs::*;
use crate::structs::*;
use crate::utils::*;

pub struct Definition {
    pub ty: Option<syn::Type>,
    pub definition: Option<TokenStream>,
    // For `Desc` types, the method body is for `describe`.
    // For `Change` types, the method body is for `comparison`.
    pub method_body: TokenStream,
}

impl quote::ToTokens for Definition {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(body) = &self.definition {
            *tokens = quote!(#body);
        }
    }
}

impl Definition {
    pub fn assoc_type(ty: &syn::Type, name: &str) -> syn::Type {
        let ident = format_ident!("{}", name);
        syn::parse2(quote!(<#ty as comparable::Comparable>::#ident))
            .unwrap_or_else(|_| panic!("Failed to parse associated type"))
    }

    pub fn changed_type(ty: &syn::Type) -> syn::Type {
        syn::parse2(quote!(comparable::Changed<#ty>))
            .unwrap_or_else(|_| panic!("Failed to parse Changed type"))
    }

    pub fn variant_name_from_field(index: usize, name: &Option<syn::Ident>) -> syn::Ident {
        if let Some(name) = name.as_ref() {
            syn::Ident::new(&name.to_string().to_case(Case::Pascal), Span::call_site())
        } else {
            format_ident!("Field{}", index)
        }
    }

    //
    // Desc associated type
    //
    // NOTE: Never called if inputs.attrs.no_description is true.
    pub fn generate_desc_type(inputs: &Inputs) -> Self {
        let type_name = &inputs.input.ident;
        let desc_name = format_ident!(
            "{}{}",
            &inputs.input.ident,
            inputs.attrs.comparable_desc_suffix
        );
        let desc_type = generate_type_definition(
            &inputs.visibility,
            &desc_name,
            &map_on_fields_over_data(true, &inputs.input.data, |r| syn::Field {
                ty: Self::assoc_type(&r.field.ty, "Desc"),
                ..r.field.clone()
            }),
        );
        Self {
            ty: Some(
                inputs
                    .attrs
                    .describe_type
                    .as_ref()
                    .unwrap_or(
                        &syn::parse2(if inputs.attrs.self_describing {
                            quote!(Self)
                        } else if inputs.attrs.compare_default {
                            quote!(Self::Change)
                        } else if let Some(ty) = &inputs.attrs.describe_type {
                            quote!(#ty)
                        } else {
                            quote!(#desc_name)
                        })
                        .expect("Failed to parse Desc type name"),
                    )
                    .clone(),
            ),
            definition: if inputs.attrs.self_describing
                || inputs.attrs.compare_default
                || inputs.attrs.describe_type.is_some()
            {
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
                .unwrap_or(if inputs.attrs.self_describing {
                    quote!(self.clone())
                } else if inputs.attrs.compare_default {
                    quote!(#type_name::default().comparison(self).unwrap_or_default())
                } else {
                    Self::generate_describe_method_body(
                        &inputs.input.ident,
                        &desc_name,
                        &inputs.input.data,
                    )
                }),
        }
    }

    //
    // describe method
    //
    fn generate_describe_method_body(
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
    // Change associated type
    //
    pub fn generate_change_type(inputs: &Inputs) -> Self {
        let type_name = &inputs.input.ident;
        let change_name = format_ident!("{}{}", type_name, inputs.attrs.comparable_change_suffix);
        let change_type =
            Self::create_change_type(&inputs.attrs, &inputs.input.ident, &inputs.input.data).map(
                |(ch_ty, helper_tys)| {
                    let ch_def = generate_type_definition(&inputs.visibility, &change_name, &ch_ty);
                    let helper_defs = helper_tys
                        .iter()
                        .map(|(name, ty)| generate_type_definition(&inputs.visibility, name, ty));
                    quote! {
                        #ch_def
                        #(#helper_defs)*
                    }
                },
            );
        Self {
            ty: if change_type.is_some() {
                (if let syn::Data::Struct(st) = &inputs.input.data {
                    match field_count(true, st.fields.iter()) {
                        0 => None,
                        1 => Some(quote!(#change_name)),
                        _ => Some(quote!(Vec<#change_name>)),
                    }
                } else {
                    Some(quote!(#change_name))
                })
                .map(|ty| syn::parse2(ty).expect("Failed to parse Change type name"))
            } else {
                None
            },
            definition: change_type,
            method_body: Self::generate_comparison_method_body(
                &inputs.attrs,
                type_name,
                &change_name,
                &inputs.input.data,
            ),
        }
    }

    fn create_change_type(
        attrs: &Attributes,
        type_name: &syn::Ident,
        data: &syn::Data,
    ) -> Option<(syn::Data, Vec<(syn::Ident, syn::Data)>)> {
        match data {
            syn::Data::Struct(st) => create_change_type_for_structs(st).map(|x| (x, Vec::new())),
            syn::Data::Enum(en) => Some(if attrs.variant_struct_fields {
                create_change_type_for_enums_with_helpers(
                    type_name,
                    &attrs.comparable_change_suffix,
                    en,
                )
            } else {
                (create_change_type_for_enums(type_name, en), Vec::new())
            }),
            syn::Data::Union(_un) => {
                panic!("comparable_derive::generate_change_type not implemented for unions")
            }
        }
    }

    //
    // comparison method
    //
    fn generate_comparison_method_body(
        attrs: &Attributes,
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
                    EnumDetails::from(attrs, type_name, change_name, en)
                        .generate_comparison_body(change_name)
                }
            }
            syn::Data::Union(_un) => {
                panic!("comparable_derive::generate_comparison_body not implemented for unions")
            }
        }
    }
}
