use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::iter::FromIterator;

use crate::attrs::*;
use crate::definition::*;
use crate::structs::*;
use crate::utils::*;

pub fn generate_describe_body_for_enums(
	type_name: &syn::Ident,
	desc_name: &syn::Ident,
	en: &syn::DataEnum,
) -> TokenStream {
	let cases = map_variants(en.variants.iter(), |variant| {
		let variant_name = &variant.ident;
		match &variant.fields {
			syn::Fields::Named(named) => {
				let (field_indices, field_names): (Vec<syn::Ident>, Vec<syn::Ident>) =
					map_fields(false, named.named.iter(), false, |r| {
						(
							format_ident!("var{}", r.index),
							r.field.ident.as_ref().expect("Found unnamed field in named struct").clone(),
						)
					})
					.into_iter()
					.unzip();
				let (field_indices_without_ignored, field_names_without_ignored): (Vec<syn::Ident>, Vec<syn::Ident>) =
					map_fields(false, named.named.iter(), true, |r| {
						(
							format_ident!("var{}", r.index),
							r.field.ident.as_ref().expect("Found unnamed field in named struct").clone(),
						)
					})
					.into_iter()
					.unzip();
				quote! {
					#type_name::#variant_name { #(#field_names: #field_indices),* } =>
					#desc_name::#variant_name { #(#field_names_without_ignored: #field_indices_without_ignored.describe()),* }
				}
			}
			syn::Fields::Unnamed(unnamed) => {
				let vars = map_fields(false, unnamed.unnamed.iter(), false, |r| format_ident!("var{}", r.index));
				let vars_without_ignored: Vec<syn::Ident> =
					map_fields(false, unnamed.unnamed.iter(), true, |r| format_ident!("var{}", r.index));

				quote! {
					#type_name::#variant_name(#(#vars),*) =>
					#desc_name::#variant_name(#(#vars_without_ignored.describe()),*)
				}
			}
			syn::Fields::Unit => {
				quote! {
					#type_name::#variant_name => #desc_name::#variant_name
				}
			}
		}
	});
	if cases.is_empty() {
		quote!(panic!("Cannot construct empty enum"))
	} else {
		quote! {
			match self {
				#(#cases),*
			}
		}
	}
}

pub fn create_change_type_for_enums(type_name: &syn::Ident, en: &syn::DataEnum) -> syn::Data {
	syn::Data::Enum(syn::DataEnum {
		variants: FromIterator::from_iter(
			map_variants(en.variants.iter(), |variant| {
				let many_fields = variant.fields.len() > 1;
				let mapped_fields = map_on_fields(false, &variant.fields, |r| syn::Field {
					ty: {
						let change_type = Definition::assoc_type(&r.field.ty, "Change");
						if many_fields {
							Definition::changed_type(&change_type)
						} else {
							change_type
						}
					},
					..r.field.clone()
				});
				if mapped_fields.is_empty() {
					None
				} else {
					Some(syn::Variant {
						ident: format_ident!("Both{}", &variant.ident),
						fields: { mapped_fields },
						..variant.clone()
					})
				}
			})
			.into_iter()
			.flatten()
			.chain(if en.variants.len() < 2 {
				vec![]
			} else {
				vec![syn::Variant {
					ident: format_ident!("Different"),
					fields: syn::Fields::Unnamed({
						let desc_field = syn::Field {
							ident: None,
							ty: Definition::assoc_type(&ident_to_type(type_name), "Desc"),
							attrs: Default::default(),
							vis: syn::Visibility::Inherited,
							colon_token: Default::default(),
						};
						syn::FieldsUnnamed {
							unnamed: FromIterator::from_iter(vec![desc_field.clone(), desc_field]),
							paren_token: Default::default(),
						}
					}),
					attrs: Default::default(),
					discriminant: Default::default(),
				}]
			}),
		),
		..*en
	})
}

// For each multi-field variant in an enum, the function below generates a
// helper [`Comparable::Change`] struct and set that variant's type for the
// enum's [`Comparable::Change`] to be `Vec<Change>`. However, we aren't using
// it quite yet. It would only really be useful if most enum variants had lots
// of fields.
pub fn create_change_type_for_enums_with_helpers(
	type_name: &syn::Ident,
	change_suffix: &syn::Ident,
	en: &syn::DataEnum,
) -> (syn::Data, Vec<(syn::Ident, syn::Data)>) {
	let mut helper_structs: Vec<(syn::Ident, syn::Data)> = Vec::new();
	let helper_structs_ref = &mut helper_structs;
	let change_type = syn::Data::Enum(syn::DataEnum {
		variants: FromIterator::from_iter(
			map_variants(en.variants.iter(), move |variant| {
				if variant.fields.is_empty() {
					None
				} else {
					let apply_change_to_field = |r: &FieldRef| syn::Field {
						ty: Definition::assoc_type(&r.field.ty, "Change"),
						..r.field.clone()
					};
					Some(syn::Variant {
						ident: format_ident!("Both{}", &variant.ident),
						fields: {
							if variant.fields.len() == 1 {
								// A map isn't needed, but it fits the pattern
								map_on_fields(false, &variant.fields, apply_change_to_field)
							} else {
								let fields_struct = &data_from_variant(variant);
								let fields_change_struct =
									create_change_type_for_structs(if let syn::Data::Struct(st) = &fields_struct {
										st
									} else {
										panic!("field_struct is not a struct!")
									})
									.unwrap();

								let fields_change_name =
									format_ident!("{}{}{}", type_name, &variant.ident, change_suffix);
								helper_structs_ref.push((fields_change_name.clone(), fields_change_struct));

								syn::Fields::Unnamed(syn::FieldsUnnamed {
									unnamed: FromIterator::from_iter(vec![syn::Field {
										ident: None,
										ty: vec_type(&ident_to_type(&fields_change_name)),
										attrs: Default::default(),
										vis: syn::Visibility::Inherited,
										colon_token: Default::default(),
									}]),
									paren_token: Default::default(),
								})
							}
						},
						..variant.clone()
					})
				}
			})
			.into_iter()
			.flatten()
			.chain(if en.variants.len() < 2 {
				vec![]
			} else {
				vec![syn::Variant {
					ident: format_ident!("Different"),
					fields: syn::Fields::Unnamed({
						let desc_field = syn::Field {
							ident: None,
							ty: Definition::assoc_type(&ident_to_type(type_name), "Desc"),
							attrs: Default::default(),
							vis: syn::Visibility::Inherited,
							colon_token: Default::default(),
						};
						syn::FieldsUnnamed {
							unnamed: FromIterator::from_iter(vec![desc_field.clone(), desc_field]),
							paren_token: Default::default(),
						}
					}),
					attrs: Default::default(),
					discriminant: Default::default(),
				}]
			}),
		),
		..*en
	});
	(change_type, helper_structs)
}

#[derive(Clone)]
struct FieldDetails {
	self_var: syn::Ident,
	other_var: syn::Ident,
	changes_var: syn::Ident,
	is_ignored: bool,
}

impl FieldDetails {
	fn from(index: usize, is_ignored: bool) -> Self {
		let self_var = format_ident!("self_var{}", index);
		let other_var = format_ident!("other_var{}", index);
		let changes_var = format_ident!("changes_var{}", index);
		FieldDetails { self_var, other_var, changes_var, is_ignored }
	}
}

#[derive(Clone)]
enum VariantFields {
	Named(Vec<(syn::Ident, FieldDetails)>),
	Unnamed(Vec<FieldDetails>),
	Unit,
}

impl VariantFields {
	fn field_names(&self) -> Option<Vec<syn::Ident>> {
		match self {
			VariantFields::Named(m) => Some(m.iter().filter_map(|(k, _)| Some(k.clone())).collect()),
			_ => None,
		}
	}

	fn field_names_filtered(&self) -> Option<Vec<syn::Ident>> {
		match self {
			VariantFields::Named(m) => {
				Some(m.iter().filter_map(|(k, l)| if !l.is_ignored { Some(k.clone()) } else { None }).collect())
			}
			_ => None,
		}
	}

	fn field_details(&self) -> Vec<FieldDetails> {
		match self {
			VariantFields::Named(m) => m.iter().map(|(_, v)| v.clone()).collect(),
			VariantFields::Unnamed(v) => v.to_vec(),
			VariantFields::Unit => Vec::new(),
		}
	}

	fn self_vars(&self) -> Vec<syn::Ident> {
		self.field_details().iter().map(|d| d.self_var.clone()).collect()
	}

	fn self_vars_filtered(&self) -> Vec<syn::Ident> {
		self.field_details()
			.iter()
			.filter_map(|d| if !d.is_ignored { Some(d.self_var.clone()) } else { None })
			.collect()
	}

	fn other_vars(&self) -> Vec<syn::Ident> {
		self.field_details().iter().map(|d| d.other_var.clone()).collect()
	}

	fn other_vars_filtered(&self) -> Vec<syn::Ident> {
		self.field_details()
			.iter()
			.filter_map(|d| if !d.is_ignored { Some(d.other_var.clone()) } else { None })
			.collect()
	}

	fn changes_vars(&self) -> Vec<syn::Ident> {
		self.field_details()
			.iter()
			.filter_map(|d| if !d.is_ignored { Some(d.changes_var.clone()) } else { None })
			.collect()
	}

	pub fn map_basic_field_info<R>(&self, mut f: impl FnMut(usize, &Option<syn::Ident>) -> R) -> Vec<R> {
		match self {
			VariantFields::Named(m) => {
				m.iter().zip(0usize..).map(|((k, _), index)| f(index, &Some(k.clone()))).collect()
			}
			VariantFields::Unnamed(v) => v.iter().zip(0usize..).map(|(_, index)| f(index, &None)).collect(),
			VariantFields::Unit => Vec::new(),
		}
	}
}

#[derive(Clone)]
struct VariantDetails {
	fields: VariantFields,
	fields_self_capture: TokenStream,
	fields_other_capture: TokenStream,
	fields_assignment: TokenStream,
	match_branch: TokenStream,
}

impl VariantDetails {
	fn from(variant: &syn::Variant) -> Self {
		let fields = match &variant.fields {
			syn::Fields::Named(named) => VariantFields::Named(
				map_fields(false, named.named.iter(), false, |r| {
					(
						r.field.ident.as_ref().expect("Unexpected unnamed field").clone(),
						FieldDetails::from(r.index, has_attr(&r.field.attrs, "comparable_ignore").is_some()),
					)
				})
				.into_iter()
				.collect(),
			),
			syn::Fields::Unnamed(unnamed) => VariantFields::Unnamed(
				map_fields(false, unnamed.unnamed.iter(), false, |r| {
					FieldDetails::from(r.index, has_attr(&r.field.attrs, "comparable_ignore").is_some())
				})
				.into_iter()
				.collect(),
			),
			syn::Fields::Unit => VariantFields::Unit,
		};

		let self_vars = fields.self_vars();
		let other_vars = fields.other_vars();
		let changes_vars = fields.changes_vars();

		if let VariantFields::Unit = fields {
			VariantDetails {
				fields,
				fields_self_capture: quote!(),
				fields_other_capture: quote!(),
				fields_assignment: quote!(),
				match_branch: Default::default(),
			}
		} else if let (Some(fields_names_without_ignored), Some(fields_names)) =
			(fields.field_names_filtered(), fields.field_names())
		{
			VariantDetails {
				fields,
				fields_self_capture: quote!({ #(#fields_names: #self_vars),* }),
				fields_other_capture: quote!({ #(#fields_names: #other_vars),* }),
				fields_assignment: quote!({ #(#fields_names_without_ignored: #changes_vars),* }),
				match_branch: Default::default(),
			}
		} else {
			VariantDetails {
				fields,
				fields_self_capture: quote!((#(#self_vars),*)),
				fields_other_capture: quote!((#(#other_vars),*)),
				fields_assignment: quote!((#(#changes_vars),*)),
				match_branch: Default::default(),
			}
		}
	}

	fn derive_match_branch(
		mut self,
		attrs: &Attributes,
		type_name: &syn::Ident,
		change_name: &syn::Ident,
		variant: &syn::Variant,
	) -> Self {
		let variant_name = &variant.ident;

		let VariantDetails { fields, fields_self_capture, fields_other_capture, fields_assignment, match_branch: _ } =
			&self;

		let both_ident = format_ident!("Both{}", variant_name);
		let self_vars_without_ignored = fields.self_vars_filtered();
		let changes_vars = fields.changes_vars();
		let other_vars_without_ignored = fields.other_vars_filtered();

		let return_result = if changes_vars.is_empty() {
			quote!(comparable::Changed::Unchanged)
		} else if fields.self_vars().len() == 1 {
			quote! {
				#(#changes_vars.map(
					|changes_var0|
					#change_name::#both_ident #fields_assignment))*
			}
		} else if attrs.variant_struct_fields {
			let fields_change_name = format_ident!("{}{}{}", type_name, variant_name, attrs.comparable_change_suffix);
			let capitalized_field_names = fields.map_basic_field_info(Definition::variant_name_from_field);
			quote! {
				let changes: Vec<#fields_change_name> = vec![
					#(#changes_vars.map(#fields_change_name::#capitalized_field_names)),*
				]
					.into_iter()
					.flatten()
					.collect();
				if changes.is_empty() {
					comparable::Changed::Unchanged
				} else {
					comparable::Changed::Changed(#change_name::#both_ident(changes))
				}
			}
		} else {
			quote! {
				if #(#changes_vars.is_unchanged())&&* {
					comparable::Changed::Unchanged
				} else {
					comparable::Changed::Changed(
						#change_name::#both_ident #fields_assignment
					)
				}
			}
		};

		self.match_branch = quote! {
			(#type_name::#variant_name #fields_self_capture,
			 #type_name::#variant_name #fields_other_capture) => {
				#(let #changes_vars = #self_vars_without_ignored.comparison(&#other_vars_without_ignored);)*
				#return_result
			}
		};
		self
	}
}

#[derive(Clone)]
pub struct EnumDetails {
	variants: Vec<VariantDetails>,
}

impl EnumDetails {
	pub fn from(attrs: &Attributes, type_name: &syn::Ident, change_name: &syn::Ident, en: &syn::DataEnum) -> Self {
		EnumDetails {
			variants: map_variants(en.variants.iter(), |variant| {
				VariantDetails::from(variant).derive_match_branch(attrs, type_name, change_name, variant)
			})
			.into_iter()
			.collect(),
		}
	}

	fn match_branches(&self) -> Vec<TokenStream> {
		self.variants.iter().map(|d| d.match_branch.clone()).collect()
	}

	pub fn generate_comparison_body(&self, change_name: &syn::Ident) -> TokenStream {
		let match_branches = self.match_branches();
		let default_case = if match_branches.len() > 1 {
			quote! {
				(_, _) => comparable::Changed::Changed(
					#change_name::Different(self.describe(), other.describe()))
			}
		} else {
			quote!()
		};
		quote! {
			match (self, other) {
				#(#match_branches),*
				#default_case
			}
		}
	}
}
