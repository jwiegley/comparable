use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use std::iter::FromIterator;

use crate::definition::*;
use crate::utils::*;

pub fn generate_describe_body_for_structs(desc_name: &syn::Ident, st: &syn::DataStruct) -> TokenStream {
	match &st.fields {
		syn::Fields::Named(named) => {
			let (field_names, field_accessors): (Vec<syn::Ident>, Vec<syn::Expr>) =
				map_fields(true, named.named.iter(), true, |r| {
					(
						r.field.ident.as_ref().expect("Found unnamed field in named struct").clone(),
						(*r.accessor)(&format_ident!("self")),
					)
				})
				.into_iter()
				.unzip();
			quote! {
				#desc_name {
					#(#field_names: #field_accessors.describe()),*
				}
			}
		}
		syn::Fields::Unnamed(unnamed) => {
			let field_indices = map_fields(false, unnamed.unnamed.iter(), true, |r| syn::Index::from(r.index));
			quote! {
				#desc_name(#(self.#field_indices.describe()),*)
			}
		}
		syn::Fields::Unit => {
			quote! { #desc_name }
		}
	}
}

pub fn create_change_type_for_structs(st: &syn::DataStruct) -> Option<syn::Data> {
	// Produce a vec that takes ignore fields into account.
	match field_count(true, st.fields.iter()) {
		0 => None,
		1 => {
			// A singleton struct is handled differently, since the only change
			// that could occur is in the single field, we only need to store that
			// change data, rather than the varying combinations that could occur
			// in the case of multiple fields.
			Some(map_on_fields_over_datastruct(true, st, |r| syn::Field {
				ty: Definition::assoc_type(&r.field.ty, "Change"),
				..r.field.clone()
			}))
		}
		_ => {
			let change_field = |r: &FieldRef| -> syn::Variant {
				let ident: syn::Ident = Definition::variant_name_from_field(r.index, &r.field.ident);
				syn::Variant {
					ident,
					fields: syn::Fields::Unnamed(syn::FieldsUnnamed {
						unnamed: FromIterator::from_iter(vec![syn::Field {
							ty: Definition::assoc_type(&r.field.ty, "Change"),
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
				syn::Fields::Named(named) => map_fields(true, named.named.iter(), true, change_field),
				syn::Fields::Unnamed(unnamed) => map_fields(true, unnamed.unnamed.iter(), true, change_field),
				syn::Fields::Unit => Vec::new(),
			};

			Some(syn::Data::Enum(syn::DataEnum {
				variants: FromIterator::from_iter(variants),
				enum_token: Default::default(),
				brace_token: Default::default(),
			}))
		}
	}
}

pub fn generate_comparison_body_for_structs(change_name: &syn::Ident, st: &syn::DataStruct) -> TokenStream {
	let (field_names_and_comparisons, field_variants): (Vec<(TokenStream, TokenStream)>, Vec<syn::Ident>) =
		map_fields(true, st.fields.iter(), true, |r: &FieldRef| -> ((TokenStream, TokenStream), syn::Ident) {
			let idx = syn::Index::from(r.index);
			let (name, variant) = if let Some(name) = r.field.ident.as_ref() {
				(quote!(#name), syn::Ident::new(&name.to_string().to_case(Case::Pascal), Span::call_site()))
			} else {
				(quote!(#idx), format_ident!("Field{}", r.index))
			};
			let self_value = (*r.accessor)(&format_ident!("self"));
			let other_value = (*r.accessor)(&format_ident!("other"));
			((name, quote!(#self_value.comparison(&#other_value))), variant)
		})
		.into_iter()
		.unzip();
	let (field_names, comparisons): (Vec<TokenStream>, Vec<TokenStream>) =
		field_names_and_comparisons.into_iter().unzip();

	if comparisons.is_empty() {
		quote!(comparable::Changed::Unchanged)
	} else if comparisons.len() == 1 {
		if let syn::Fields::Unnamed(_) = st.fields {
			quote! {
				#(#comparisons.map(#change_name))*
			}
		} else {
			quote! {
				#(#comparisons.map(|x| #change_name { #field_names: x }))*
			}
		}
	} else {
		quote! {
			let changes: Vec<#change_name> = vec![
				#(#comparisons.map(#change_name::#field_variants)),*
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
