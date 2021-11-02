use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use std::iter::FromIterator;

use crate::definition::*;
use crate::utils::*;

pub fn generate_describe_body_for_structs(
    desc_name: &syn::Ident,
    st: &syn::DataStruct,
) -> TokenStream {
    match &st.fields {
        syn::Fields::Named(named) => {
            let field_names = map_fields(named.named.iter(), |_, field| {
                field
                    .ident
                    .as_ref()
                    .expect("Found unnamed field in named struct")
            });
            quote! {
                #desc_name {
                    #(#field_names: self.#field_names.describe()),*
                }
            }
        }
        syn::Fields::Unnamed(unnamed) => {
            let field_indices =
                map_fields(unnamed.unnamed.iter(), |index, _| syn::Index::from(index));
            quote! {
                #desc_name(#(self.#field_indices.describe()),*)
            }
        }
        syn::Fields::Unit => {
            quote! { #desc_name }
        }
    }
}

pub fn create_change_type_for_structs(st: &syn::DataStruct) -> syn::Data {
    if is_datastruct_with_many_fields(st) {
        let change_field = |index: usize, field: &syn::Field| -> syn::Variant {
            let ident: syn::Ident = if let Some(name) = field.ident.as_ref() {
                syn::Ident::new(&name.to_string().to_case(Case::Pascal), Span::call_site())
            } else {
                format_ident!("Field{}", index)
            };
            syn::Variant {
                ident,
                fields: syn::Fields::Unnamed(syn::FieldsUnnamed {
                    unnamed: FromIterator::from_iter(vec![syn::Field {
                        ty: Definition::assoc_type(&field.ty, "Change"),
                        attrs: Default::default(),
                        vis: syn::Visibility::Inherited,
                        ident: Default::default(),
                        colon_token: Default::default(),
                    }]),
                    paren_token: Default::default(),
                }),
                attrs: Default::default(),
                discriminant: Default::default(),
            }
        };

        let variants = match &st.fields {
            syn::Fields::Named(named) => map_fields(named.named.iter(), change_field),
            syn::Fields::Unnamed(unnamed) => map_fields(unnamed.unnamed.iter(), change_field),
            syn::Fields::Unit => Vec::new(),
        };

        syn::Data::Enum(syn::DataEnum {
            variants: FromIterator::from_iter(variants),
            enum_token: Default::default(),
            brace_token: Default::default(),
        })
    } else {
        // A singleton struct is handled differently, since the only change
        // that could occur is in the single field, we only need to store that
        // change data, rather than the varying combinations that could occur
        // in the case of multiple fields.
        map_on_fields_over_datastruct(st, |_, field| syn::Field {
            ty: Definition::assoc_type(&field.ty, "Change"),
            ..field.clone()
        })
    }
}

pub fn generate_comparison_body_for_structs(
    change_name: &syn::Ident,
    st: &syn::DataStruct,
) -> TokenStream {
    let inspect_field = |index: usize, field: &syn::Field| -> (TokenStream, syn::Ident) {
        let idx = syn::Index::from(index);
        if let Some(name) = field.ident.as_ref() {
            (
                quote!(#name),
                syn::Ident::new(&name.to_string().to_case(Case::Pascal), Span::call_site()),
            )
        } else {
            (quote!(#idx), format_ident!("Field{}", index))
        }
    };

    let (field_names, field_variants): (Vec<TokenStream>, Vec<syn::Ident>) = match &st.fields {
        syn::Fields::Named(named) => map_fields(named.named.iter(), inspect_field)
            .into_iter()
            .unzip(),
        syn::Fields::Unnamed(unnamed) => map_fields(unnamed.unnamed.iter(), inspect_field)
            .into_iter()
            .unzip(),
        syn::Fields::Unit => (Vec::new(), Vec::new()),
    };

    if field_names.is_empty() {
        quote!(comparable::Changed::Unchanged)
    } else if field_names.len() == 1 {
        quote! {
            #(self.#field_names.comparison(&other.#field_names).map(#change_name))*
        }
    } else {
        quote! {
            let changes: Vec<#change_name> = vec![
                #(self.#field_names.comparison(&other.#field_names)
                      .map(#change_name::#field_variants)),*
            ]
                .into_iter()
                .flatten()
                .collect();
            if changes.is_empty() {
                comparable::Changed::Unchanged
            } else {
                comparable::Changed::Changed(changes)
            }
        }
    }
}
