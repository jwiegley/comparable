use proc_macro2::TokenStream;
use quote::quote;
use std::iter::FromIterator;

pub fn unit_type() -> syn::Type {
    syn::Type::Tuple(syn::TypeTuple {
        paren_token: syn::token::Paren {
            span: proc_macro2::Span::call_site(),
        },
        elems: syn::punctuated::Punctuated::new(),
    })
}

pub fn has_attr<'a>(attrs: &'a [syn::Attribute], attr_name: &str) -> Option<&'a syn::Attribute> {
    attrs.iter().find(|attr| attr.path.is_ident(attr_name))
}

pub fn is_datastruct_with_many_fields(st: &syn::DataStruct) -> bool {
    match &st.fields {
        syn::Fields::Named(named) => named.named.len() > 1,
        syn::Fields::Unnamed(unnamed) => unnamed.unnamed.len() > 1,
        syn::Fields::Unit => false,
    }
}

pub fn is_struct_with_many_fields(data: &syn::Data) -> bool {
    match data {
        syn::Data::Struct(st) => is_datastruct_with_many_fields(st),
        _ => false,
    }
}

pub fn _variant_to_datastruct(variant: &syn::Variant) -> syn::DataStruct {
    syn::DataStruct {
        fields: variant.fields.clone(),
        struct_token: Default::default(),
        semi_token: Default::default(),
    }
}

pub fn map_on_fields_over_data(
    data: &syn::Data,
    f: impl Fn(usize, &syn::Field) -> syn::Field + Copy,
) -> syn::Data {
    match data {
        syn::Data::Struct(st) => map_on_fields_over_datastruct(st, f),
        syn::Data::Enum(en) => syn::Data::Enum(syn::DataEnum {
            variants: FromIterator::from_iter(map_variants(&en.variants, move |v| syn::Variant {
                fields: map_on_fields(&v.fields, f),
                ..v.clone()
            })),
            ..*en
        }),
        syn::Data::Union(un) => syn::Data::Union(syn::DataUnion {
            fields: syn::FieldsNamed {
                named: FromIterator::from_iter(map_fields(un.fields.named.iter(), f)),
                ..un.fields.clone()
            },
            ..*un
        }),
    }
}

pub fn map_on_fields_over_datastruct(
    st: &syn::DataStruct,
    f: impl Fn(usize, &syn::Field) -> syn::Field,
) -> syn::Data {
    syn::Data::Struct(syn::DataStruct {
        fields: map_on_fields(&st.fields, f),
        ..*st
    })
}

pub fn _map_on_variants_over_dataenum(
    en: &syn::DataEnum,
    f: impl Fn(&syn::Variant) -> syn::Variant,
) -> syn::Data {
    syn::Data::Enum(syn::DataEnum {
        variants: FromIterator::from_iter(map_variants(en.variants.iter(), f)),
        ..*en
    })
}

pub fn map_on_fields(
    fields: &syn::Fields,
    f: impl Fn(usize, &syn::Field) -> syn::Field,
) -> syn::Fields {
    match fields {
        syn::Fields::Named(named) => syn::Fields::Named(syn::FieldsNamed {
            named: FromIterator::from_iter(map_fields(named.named.iter(), f)),
            ..*named
        }),
        syn::Fields::Unnamed(unnamed) => syn::Fields::Unnamed(syn::FieldsUnnamed {
            unnamed: FromIterator::from_iter(map_fields(unnamed.unnamed.iter(), f)),
            ..*unnamed
        }),
        syn::Fields::Unit => syn::Fields::Unit,
    }
}

pub fn map_fields<'a, R>(
    fields: impl IntoIterator<Item = &'a syn::Field>,
    f: impl Fn(usize, &'a syn::Field) -> R,
) -> Vec<R> {
    fields
        .into_iter()
        .zip(0usize..)
        .map(|(field, index)| {
            if has_attr(&field.attrs, "comparable_ignore").is_none() {
                Some(f(index, field))
            } else {
                None
            }
        })
        .flatten()
        .collect()
}

pub fn map_variants<'a, R>(
    variants: impl IntoIterator<Item = &'a syn::Variant>,
    f: impl Fn(&syn::Variant) -> R,
) -> Vec<R> {
    variants
        .into_iter()
        .map(|variant| {
            if has_attr(&variant.attrs, "comparable_ignore").is_none() {
                Some(f(variant))
            } else {
                None
            }
        })
        .flatten()
        .collect()
}

pub fn generate_type_definition(
    visibility: &syn::Visibility,
    type_name: &syn::Ident,
    data: &syn::Data,
) -> TokenStream {
    let (keyword, body) = match data {
        syn::Data::Struct(st) => (
            quote!(struct),
            match &st.fields {
                syn::Fields::Named(named) => {
                    let fields = map_fields(named.named.iter(), |_, field| {
                        let ident = field
                            .ident
                            .as_ref()
                            .expect("Found unnamed field in named struct");
                        let ty = &field.ty;
                        quote!(#ident: #ty)
                    });
                    quote! {
                        {
                            #(#fields),*
                        }
                    }
                }
                syn::Fields::Unnamed(unnamed) => {
                    let field_types =
                        map_fields(unnamed.unnamed.iter(), |_, field| field.ty.clone());
                    quote! {
                        (#(#field_types),*);
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
                        let fields = map_fields(named.named.iter(), |_, field| {
                            let ident = field
                                .ident
                                .as_ref()
                                .expect("Found unnamed field in named struct");
                            let ty = &field.ty;
                            quote!(#ident: #ty)
                        });
                        quote! {
                            #variant_name { #(#fields),* }
                        }
                    }
                    syn::Fields::Unnamed(unnamed) => {
                        let field_types =
                            map_fields(unnamed.unnamed.iter(), |_, field| field.ty.clone());
                        quote! {
                            #variant_name(#(#field_types),*)
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
    quote! {
        // #[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
        #[derive(PartialEq, Debug)]
        #visibility #keyword #type_name#body
    }
}
