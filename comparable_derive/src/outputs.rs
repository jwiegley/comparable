use proc_macro2::TokenStream;
use quote::quote;

use crate::definition::*;
use crate::inputs::*;
use crate::utils::*;

pub struct Outputs {
	pub desc: Option<Definition>,
	pub change: Option<Definition>,
}

impl Outputs {
	pub fn generate(self, inputs: &Inputs) -> TokenStream {
		let Outputs { desc, change } = self;

		let impl_comparable = Self::impl_comparable(
			&inputs.input.ident,
			&inputs.input.generics,
			desc.as_ref().and_then(|d| d.ty.as_ref()).unwrap_or(&unit_type()),
			desc.as_ref().map(|d| &d.method_body).unwrap_or(&quote!()),
			change.as_ref().and_then(|c| c.ty.as_ref()).unwrap_or(&unit_type()),
			change.as_ref().map(|c| &c.method_body).unwrap_or(&quote!(comparable::Changed::Unchanged)),
		);

		#[allow(unused_variables)] // compiler doesn't see the use of x
		let desc = desc.map(|x| quote!(#x)).unwrap_or_default();
		#[allow(unused_variables)] // compiler doesn't see the use of x
		let change = change.map(|x| quote!(#x)).unwrap_or_default();

		quote! {
			#desc
			#change
			#impl_comparable
		}
	}

	fn impl_comparable(
		name: &syn::Ident,
		generics: &syn::Generics,
		describe_type: &syn::Type,
		describe_body: &TokenStream,
		change_type: &syn::Type,
		change_body: &TokenStream,
	) -> TokenStream {
		let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

		// Build where clause predicates for T: Comparable for each generic type parameter
		let mut where_predicates = vec![];
		for param in &generics.params {
			if let syn::GenericParam::Type(type_param) = param {
				let ident = &type_param.ident;
				where_predicates.push(quote!(#ident: comparable::Comparable));
			}
		}

		let extended_where_clause = if !where_predicates.is_empty() || where_clause.is_some() {
			let existing_predicates = where_clause.map(|w| {
				let predicates = &w.predicates;
				quote!(#predicates,)
			});
			quote! {
				where #existing_predicates #(#where_predicates),*
			}
		} else {
			quote!()
		};

		quote! {
			impl #impl_generics comparable::Comparable for #name #ty_generics #extended_where_clause {
				type Desc = #describe_type;
				fn describe(&self) -> Self::Desc {
					#describe_body
				}

				type Change = #change_type;
				fn comparison(&self, other: &Self) -> comparable::Changed<Self::Change> {
					#change_body
				}
			}
		}
	}
}
