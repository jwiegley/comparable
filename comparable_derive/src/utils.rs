use proc_macro2::TokenStream;
use quote::quote;
use std::collections::BTreeMap;
use std::iter::FromIterator;

pub fn unit_type() -> syn::Type {
	syn::Type::Tuple(syn::TypeTuple {
		paren_token: syn::token::Paren { span: proc_macro2::Span::call_site() },
		elems: syn::punctuated::Punctuated::new(),
	})
}

pub fn ident_to_type(ident: &syn::Ident) -> syn::Type {
	syn::parse2(quote!(#ident)).unwrap_or_else(|_| panic!("Failed to parse type"))
}

#[allow(dead_code)]
pub fn vec_type(ty: &syn::Type) -> syn::Type {
	syn::parse2(quote!(Vec<#ty>)).unwrap_or_else(|_| panic!("Failed to parse Vec type"))
}

pub fn has_attr<'a>(attrs: &'a [syn::Attribute], attr_name: &str) -> Option<&'a syn::Attribute> {
	attrs.iter().find(|attr| attr.path.is_ident(attr_name))
}

#[allow(dead_code)]
pub fn data_from_variant(variant: &syn::Variant) -> syn::Data {
	syn::Data::Struct(syn::DataStruct {
		fields: variant.fields.clone(),
		struct_token: Default::default(),
		semi_token: Default::default(),
	})
}

pub fn map_on_fields_over_data(
	inject_synthetics: bool,
	data: &syn::Data,
	f: impl FnMut(&FieldRef) -> syn::Field + Copy,
) -> syn::Data {
	match data {
		syn::Data::Struct(st) => map_on_fields_over_datastruct(inject_synthetics, st, f),
		syn::Data::Enum(en) => syn::Data::Enum(syn::DataEnum {
			variants: FromIterator::from_iter(map_variants(&en.variants, move |v| syn::Variant {
				fields: map_on_fields(inject_synthetics, &v.fields, f),
				..v.clone()
			})),
			..*en
		}),
		syn::Data::Union(un) => syn::Data::Union(syn::DataUnion {
			fields: syn::FieldsNamed {
				named: FromIterator::from_iter(map_fields(inject_synthetics, un.fields.named.iter(), f)),
				..un.fields.clone()
			},
			..*un
		}),
	}
}

pub fn map_on_fields_over_datastruct(
	inject_synthetics: bool,
	st: &syn::DataStruct,
	f: impl FnMut(&FieldRef) -> syn::Field,
) -> syn::Data {
	syn::Data::Struct(syn::DataStruct { fields: map_on_fields(inject_synthetics, &st.fields, f), ..*st })
}

pub fn _map_on_variants_over_dataenum(en: &syn::DataEnum, f: impl FnMut(&syn::Variant) -> syn::Variant) -> syn::Data {
	syn::Data::Enum(syn::DataEnum { variants: FromIterator::from_iter(map_variants(en.variants.iter(), f)), ..*en })
}

pub fn map_on_fields(
	inject_synthetics: bool,
	fields: &syn::Fields,
	f: impl FnMut(&FieldRef) -> syn::Field,
) -> syn::Fields {
	match fields {
		syn::Fields::Named(named) => syn::Fields::Named(syn::FieldsNamed {
			named: FromIterator::from_iter(map_fields(inject_synthetics, named.named.iter(), f)),
			..*named
		}),
		syn::Fields::Unnamed(unnamed) => syn::Fields::Unnamed(syn::FieldsUnnamed {
			unnamed: FromIterator::from_iter(map_fields(inject_synthetics, unnamed.unnamed.iter(), f)),
			..*unnamed
		}),
		syn::Fields::Unit => syn::Fields::Unit,
	}
}

pub struct FieldRef<'a> {
	pub index: usize,
	pub field: &'a syn::Field,
	pub accessor: Box<dyn Fn(&syn::Ident) -> syn::Expr>,
}

fn standard_accessor(index: usize, field: &syn::Field) -> Box<dyn Fn(&syn::Ident) -> syn::Expr> {
	let ident = match &field.ident {
		None => {
			let idx = syn::Index::from(index);
			quote!(#idx)
		}
		Some(ident) => quote!(#ident),
	};
	Box::new(move |x| syn::parse2(quote!(#x.#ident)).expect("Could not create standard accessor!"))
}

pub fn map_fields<'a, 'b: 'a, R>(
	inject_synthetics: bool,
	fields: impl IntoIterator<Item = &'a syn::Field>,
	mut f: impl FnMut(&FieldRef) -> R,
) -> Vec<R> {
	let mut index = 0;
	let mut result = Vec::new();
	fields.into_iter().for_each(|field| {
		if inject_synthetics {
			if let Some(synthetics) = has_attr(&field.attrs, "comparable_synthetic").map(|attr| {
				parse_synthetics(&attr.tokens)
					.expect("Argument to comparable_synthetic must be a set of field values in braces")
			}) {
				synthetics.into_iter().for_each(|(ident, closure)| {
					result.push(f(&FieldRef {
						index,
						field: &syn::Field {
							ident: Some(ident),
							ty: match &closure.output {
								syn::ReturnType::Default => unit_type(),
								syn::ReturnType::Type(_, ty) => ty.as_ref().clone(),
							},
							attrs: Default::default(),
							vis: syn::Visibility::Inherited,
							colon_token: Default::default(),
						},
						accessor: Box::new(move |x| {
							syn::parse2(quote!((#closure)(&#x))).expect("Could not create synthetic accessor!")
						}),
					}));
					index += 1;
				});
			}
		}
		if has_attr(&field.attrs, "comparable_ignore").is_none() {
			result.push(f(&FieldRef { index, field, accessor: standard_accessor(index, field) }));
			index += 1;
		}
	});
	result
}

pub fn field_count<'a>(inject_synthetics: bool, fields: impl IntoIterator<Item = &'a syn::Field>) -> usize {
	map_fields(inject_synthetics, fields, |_| ()).len()
}

pub fn map_variants<'a, R>(
	variants: impl IntoIterator<Item = &'a syn::Variant>,
	f: impl FnMut(&syn::Variant) -> R,
) -> Vec<R> {
	variants.into_iter().map(f).collect()
}

fn parse_synthetics(tokens: &TokenStream) -> Result<BTreeMap<syn::Ident, syn::ExprClosure>, syn::Error> {
	let block: syn::Block = syn::parse2(tokens.clone())?;
	Ok(block
		.stmts
		.into_iter()
		.map(|s| {
			if let syn::Stmt::Local(syn::Local {
				attrs: _,
				let_token: _,
				pat: syn::Pat::Ident(syn::PatIdent { attrs: _, by_ref: _, mutability: _, ident, subpat: _ }),
				init: Some((_, expr)),
				semi_token: _,
			}) = s
			{
				if let syn::Expr::Closure(closure) = *expr {
					(ident, closure)
				} else {
					panic!("Let values in comparable_synthetic must be fully typed closures")
				}
			} else {
				panic!("Syntax error parsing argument to comparable_synthetic")
			}
		})
		.collect())
}

pub fn generate_type_definition(visibility: &syn::Visibility, type_name: &syn::Ident, data: &syn::Data) -> TokenStream {
	let (keyword, body) = match data {
		syn::Data::Struct(st) => (
			quote!(struct),
			match &st.fields {
				syn::Fields::Named(named) => {
					let fields = map_fields(false, named.named.iter(), |r| {
						let vis = &r.field.vis;
						let ident = r.field.ident.as_ref().expect("Found unnamed field in named struct");
						let ty = &r.field.ty;
						quote!(#vis #ident: #ty)
					});
					quote! {
						{
							#(#fields),*
						}
					}
				}
				syn::Fields::Unnamed(unnamed) => {
					let (field_vis, field_types): (Vec<syn::Visibility>, Vec<syn::Type>) =
						map_fields(false, unnamed.unnamed.iter(), |r| (r.field.vis.clone(), r.field.ty.clone()))
							.into_iter()
							.unzip();
					quote! {
						(#(#field_vis #field_types),*);
					}
				}
				syn::Fields::Unit => {
					quote! { ; }
				}
			},
		),
		syn::Data::Enum(en) => (quote!(enum), {
			let variants = map_variants(en.variants.iter(), |variant| {
				let variant_name = &variant.ident;
				match &variant.fields {
					syn::Fields::Named(named) => {
						let fields = map_fields(false, named.named.iter(), |r| {
							let vis = &r.field.vis;
							let ident = r.field.ident.as_ref().expect("Found unnamed field in named struct");
							let ty = &r.field.ty;
							quote!(#vis #ident: #ty)
						});
						quote! {
							#variant_name { #(#fields),* }
						}
					}
					syn::Fields::Unnamed(unnamed) => {
						let (field_vis, field_types): (Vec<syn::Visibility>, Vec<syn::Type>) =
							map_fields(false, unnamed.unnamed.iter(), |r| (r.field.vis.clone(), r.field.ty.clone()))
								.into_iter()
								.unzip();
						quote! {
							#variant_name(#(#field_vis #field_types),*)
						}
					}
					syn::Fields::Unit => {
						quote! {
							#variant_name
						}
					}
				}
			});
			quote! {
				{
					#(#variants),*
				}
			}
		}),
		syn::Data::Union(_un) => {
			panic!("comparable_derive::generate_type_definition not implemented for unions")
		}
	};
	let derive_serde = if cfg!(feature = "serde") {
		quote! {
			#[derive(serde::Serialize, serde::Deserialize)]
		}
	} else {
		quote! {}
	};
	quote! {
		#derive_serde
		#[derive(PartialEq, Debug)]
		#visibility #keyword #type_name#body
	}
}
