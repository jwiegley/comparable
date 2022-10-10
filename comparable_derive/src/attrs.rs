use quote::format_ident;

use crate::utils::has_attr;

pub struct Attributes {
	pub describe_type: Option<syn::Type>,
	pub describe_body: Option<syn::Expr>,
	pub no_description: bool,
	pub self_describing: bool,
	pub variant_struct_fields: bool,
	pub compare_default: bool,
	pub comparable_public: bool,
	pub comparable_private: bool,
	pub comparable_desc_suffix: syn::Ident,
	pub comparable_change_suffix: syn::Ident,
}

impl Attributes {
	pub fn from(attrs: &[syn::Attribute]) -> Self {
		Attributes {
			describe_type: has_attr(attrs, "describe_type")
				.map(|x| x.parse_args::<syn::Type>().expect("Failed to parse \"describe_type\" attribute")),
			describe_body: has_attr(attrs, "describe_body")
				.map(|x| x.parse_args::<syn::Expr>().expect("Failed to parse \"describe_body\" attribute")),

			no_description: has_attr(attrs, "no_description").is_some(),
			self_describing: has_attr(attrs, "self_describing").is_some(),

			variant_struct_fields: has_attr(attrs, "variant_struct_fields").is_some(),
			compare_default: has_attr(attrs, "compare_default").is_some(),

			comparable_public: has_attr(attrs, "comparable_public").is_some(),
			comparable_private: has_attr(attrs, "comparable_private").is_some(),

			comparable_desc_suffix: attr_to_ident(attrs, "comparable_desc_suffix", "Desc"),
			comparable_change_suffix: attr_to_ident(attrs, "comparable_change_suffix", "Change"),
		}
	}
}

fn attr_to_ident(attrs: &[syn::Attribute], name: &str, suffix: &str) -> syn::Ident {
	has_attr(attrs, name)
		.map(|x| x.parse_args::<syn::Ident>().unwrap_or_else(|_| panic!("Failed to parse \"{}\" attribute", name)))
		.unwrap_or_else(|| format_ident!("{}", suffix))
}
