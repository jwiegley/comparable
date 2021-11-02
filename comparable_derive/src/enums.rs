use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashMap;
use std::iter::FromIterator;

use crate::definition::*;
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
                    map_fields(named.named.iter(), |index, field| {
                        (
                            format_ident!("var{}", index),
                            field
                                .ident
                                .as_ref()
                                .expect("Found unnamed field in named struct")
                                .clone(),
                        )
                    })
                    .into_iter()
                    .unzip();
                quote! {
                    #type_name::#variant_name { #(#field_names: #field_indices),* } =>
                    #desc_name::#variant_name { #(#field_names: #field_indices.describe()),* }
                }
            }
            syn::Fields::Unnamed(unnamed) => {
                let vars = map_fields(unnamed.unnamed.iter(), |index, _| {
                    format_ident!("var{}", index)
                });
                quote! {
                    #type_name::#variant_name(#(#vars),*) =>
                    #desc_name::#variant_name(#(#vars.describe()),*)
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
                if variant.fields.is_empty() {
                    None
                } else {
                    Some(syn::Variant {
                        ident: format_ident!("Both{}", &variant.ident),
                        fields: {
                            let many_fields = variant.fields.len() > 1;
                            map_on_fields(&variant.fields, |_, field| syn::Field {
                                ty: {
                                    let change_type = Definition::assoc_type(&field.ty, "Change");
                                    if many_fields {
                                        Definition::changed_type(&change_type)
                                    } else {
                                        change_type
                                    }
                                },
                                ..field.clone()
                            })
                        },
                        ..variant.clone()
                    })
                }
            })
            .into_iter()
            .flatten()
            .into_iter()
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

pub fn _create_change_type_for_enums_with_helpers(
    type_name: &syn::Ident,
    en: &syn::DataEnum,
) -> syn::Data {
    let mut helper_structs: HashMap<syn::Ident, syn::Data> = HashMap::new();
    syn::Data::Enum(syn::DataEnum {
        variants: FromIterator::from_iter(
            map_variants(en.variants.iter(), move |variant| {
                if variant.fields.is_empty() {
                    None
                } else {
                    let apply_change_to_field = |_, field: &syn::Field| syn::Field {
                        ty: Definition::assoc_type(&field.ty, "Change"),
                        ..field.clone()
                    };
                    Some(syn::Variant {
                        ident: format_ident!("Both{}", &variant.ident),
                        fields: {
                            if variant.fields.len() == 1 {
                                // A map isn't needed, but it fits the pattern
                                map_on_fields(&variant.fields, apply_change_to_field)
                            } else {
                                let fields_change_struct = map_on_fields_over_data(
                                    &data_from_variant(variant),
                                    apply_change_to_field,
                                );
                                let fields_change_name =
                                    format_ident!("{}{}Change", type_name, &variant.ident);
                                helper_structs
                                    .insert(fields_change_name.clone(), fields_change_struct);
                                syn::Fields::Unnamed(syn::FieldsUnnamed {
                                    unnamed: FromIterator::from_iter(
                                        vec![syn::Field {
                                            ident: None,
                                            ty: vec_type(&ident_to_type(&fields_change_name)),
                                            attrs: Default::default(),
                                            vis: syn::Visibility::Inherited,
                                            colon_token: Default::default(),
                                        }]
                                        .into_iter(),
                                    ),
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
            .into_iter()
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

#[derive(Clone)]
struct FieldDetails {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    self_var: syn::Ident,
    other_var: syn::Ident,
    changes_var: syn::Ident,
    let_comparison: TokenStream,
}

impl FieldDetails {
    fn from(index: usize, field: &syn::Field) -> Self {
        let self_var = format_ident!("self_var{}", index);
        let other_var = format_ident!("other_var{}", index);
        let changes_var = format_ident!("changes_var{}", index);
        let let_comparison = quote! {
            let #changes_var = #self_var.comparison(&#other_var);
        };
        FieldDetails {
            ident: field.ident.clone(),
            ty: field.ty.clone(),
            self_var,
            other_var,
            changes_var,
            let_comparison,
        }
    }
}

#[derive(Clone)]
enum VariantFields {
    Named(HashMap<syn::Ident, FieldDetails>),
    Unnamed(Vec<FieldDetails>),
    Unit,
}

impl VariantFields {
    fn field_names(&self) -> Option<Vec<syn::Ident>> {
        match self {
            VariantFields::Named(m) => Some(m.keys().cloned().into_iter().collect()),
            _ => None,
        }
    }

    fn field_details(&self) -> Vec<FieldDetails> {
        match self {
            VariantFields::Named(m) => m.values().cloned().into_iter().collect(),
            VariantFields::Unnamed(v) => v.to_vec(),
            VariantFields::Unit => Vec::new(),
        }
    }

    fn self_vars(&self) -> Vec<syn::Ident> {
        self.field_details()
            .iter()
            .map(|d| d.self_var.clone())
            .collect()
    }

    fn other_vars(&self) -> Vec<syn::Ident> {
        self.field_details()
            .iter()
            .map(|d| d.other_var.clone())
            .collect()
    }

    fn changes_vars(&self) -> Vec<syn::Ident> {
        self.field_details()
            .iter()
            .map(|d| d.changes_var.clone())
            .collect()
    }

    fn let_comparisons(&self) -> Vec<TokenStream> {
        self.field_details()
            .iter()
            .map(|d| d.let_comparison.clone())
            .collect()
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
                map_fields(named.named.iter(), |index, field| {
                    (
                        field
                            .ident
                            .as_ref()
                            .expect("Unexpected unnamed field")
                            .clone(),
                        FieldDetails::from(index, field),
                    )
                })
                .into_iter()
                .collect(),
            ),
            syn::Fields::Unnamed(unnamed) => VariantFields::Unnamed(
                map_fields(unnamed.unnamed.iter(), |index, field| {
                    FieldDetails::from(index, field)
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
        } else if let Some(fields_names) = fields.field_names() {
            VariantDetails {
                fields,
                fields_self_capture: quote!({ #(#fields_names: #self_vars),* }),
                fields_other_capture: quote!({ #(#fields_names: #other_vars),* }),
                fields_assignment: quote!({ #(#fields_names: #changes_vars),* }),
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
        type_name: &syn::Ident,
        change_name: &syn::Ident,
        variant: &syn::Variant,
    ) -> Self {
        let variant_name = &variant.ident;

        let VariantDetails {
            fields,
            fields_self_capture,
            fields_other_capture,
            fields_assignment,
            match_branch: _,
        } = &self;

        let both_ident = format_ident!("Both{}", variant_name);
        let changes_vars = fields.changes_vars();

        let return_result = if changes_vars.is_empty() {
            quote!(comparable::Changed::Unchanged)
        } else if changes_vars.len() == 1 {
            quote! {
                #(#changes_vars.map(
                    |changes_var0|
                    #change_name::#both_ident #fields_assignment))*
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

        let let_comparisons = fields.let_comparisons();
        self.match_branch = quote! {
            (#type_name::#variant_name #fields_self_capture,
             #type_name::#variant_name #fields_other_capture) => {
                #(#let_comparisons)*
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
    pub fn from(type_name: &syn::Ident, change_name: &syn::Ident, en: &syn::DataEnum) -> Self {
        EnumDetails {
            variants: map_variants(en.variants.iter(), |variant| {
                VariantDetails::from(variant).derive_match_branch(type_name, change_name, variant)
            })
            .into_iter()
            .collect(),
        }
    }

    fn match_branches(&self) -> Vec<TokenStream> {
        self.variants
            .iter()
            .map(|d| d.match_branch.clone())
            .collect()
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
