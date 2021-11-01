use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use std::iter::FromIterator;

fn has_attr<'a>(attrs: &'a [syn::Attribute], attr_name: &str) -> Option<&'a syn::Attribute> {
    attrs
        .iter()
        .filter(|attr| attr.path.is_ident(attr_name))
        .next()
}

#[proc_macro_derive(
    Delta,
    attributes(
        describe_type,
        describe_body,
        no_description,
        compare_default,
        delta_public,
        delta_private,
        delta_ignore
    )
)]
pub fn delta_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let inputs: Inputs = Inputs::from(&input);
    let outputs: Outputs = inputs.process_data();
    let tokens: TokenStream = outputs.generate(&inputs);
    tokens.into()
}

struct Inputs<'a> {
    attrs: Attributes,
    input: &'a syn::DeriveInput,
    visibility: syn::Visibility,
}

impl<'a> Inputs<'a> {
    fn from(input: &'a syn::DeriveInput) -> Self {
        let attrs = Attributes::from(&input.attrs);

        let visibility = if attrs.delta_private {
            syn::Visibility::Inherited
        } else if attrs.delta_public {
            syn::Visibility::Public(syn::VisPublic {
                pub_token: syn::token::Pub {
                    span: Span::call_site(),
                },
            })
        } else {
            input.vis.clone()
        };

        Inputs {
            attrs,
            input,
            visibility,
        }
    }
}

struct Attributes {
    describe_type: Option<syn::Type>,
    describe_body: Option<syn::Expr>,
    no_description: bool,
    compare_default: bool,
    delta_public: bool,
    delta_private: bool,
}

impl Attributes {
    fn from(attrs: &[syn::Attribute]) -> Self {
        /*
                let desc_type = if has_attr(&input.attrs, "no_description").is_some() {
                    quote!(())
                } else if let Some(ty) = has_attr(&input.attrs, "describe_type").map(|x| {
                    x.parse_args::<syn::Type>()
                        .expect("Failed to parse \"describe_type\" attribute")
                        .into_token_stream()
                }) {
                    ty
                } else if compare_default {
                    quote!(Self::Change)
                } else {
                    quote!(Self)
                };

                let desc_body = if has_attr(&input.attrs, "no_description").is_some() {
                    quote!(())
                } else if let Some(body) = has_attr(&input.attrs, "describe_body").map(|x| {
                    x.parse_args::<syn::Expr>()
                        .expect("Failed to parse \"describe_body\" attribute")
                        .into_token_stream()
                }) {
                    body
                } else if compare_default {
                    quote!(#type_name::default().delta(self).unwrap_or_default())
                } else {
                    quote!((*self).clone())
                };
        */

        Attributes {
            describe_type: has_attr(attrs, "describe_type").map(|x| {
                x.parse_args::<syn::Type>()
                    .expect("Failed to parse \"describe_type\" attribute")
            }),
            describe_body: has_attr(attrs, "describe_body").map(|x| {
                x.parse_args::<syn::Expr>()
                    .expect("Failed to parse \"describe_body\" attribute")
            }),
            no_description: has_attr(attrs, "no_description").is_some(),
            compare_default: has_attr(attrs, "compare_default").is_some(),
            delta_public: has_attr(attrs, "delta_public").is_some(),
            delta_private: has_attr(attrs, "delta_private").is_some(),
        }
    }
}

struct Definition {
    ty: syn::Type,
    definition: TokenStream,
    // For `Desc` types, the method body is for `describe`.
    // For `Change` types, the method body is for `delta`.
    method_body: TokenStream,
    impl_section: Option<TokenStream>,
}

impl Definition {
    fn assoc_type(ty: &syn::Type, name: &str) -> TokenStream {
        let ident = format_ident!("{}", name);
        quote!(<#ty as delta::Delta>::#ident)
    }

    fn generate_desc_from_data(inputs: &Inputs) -> Self {
        let desc_name = format_ident!("{}Desc", &inputs.input.ident);
        let desc = Self::generate_type_definition(
            &inputs.visibility,
            &desc_name,
            &map_on_types_of_fields_over_data(&inputs.input.data, |ty| {
                syn::parse2(Self::assoc_type(ty, "Desc"))
                    .expect(&format!("Failed to parse associated type for Desc"))
            }),
        );
        Self {
            ty: syn::parse2(quote!(#desc_name)).expect("Failed to parse Desc type name"),
            definition: quote! {
                #[derive(Clone, Debug, PartialEq)]
                #desc
            },
            method_body: Self::generate_describe_body(
                &inputs.input.ident,
                &desc_name,
                &inputs.input.data,
            ),
            impl_section: None,
        }
    }

    fn generate_describe_body(
        type_name: &syn::Ident,
        desc_name: &syn::Ident,
        data: &syn::Data,
    ) -> TokenStream {
        match data {
            syn::Data::Struct(st) => match &st.fields {
                syn::Fields::Named(named) => {
                    let field_names = map_fields(named.named.iter(), |_, field| {
                        field
                            .ident
                            .as_ref()
                            .expect("Found unnamed field in named struct")
                            .clone()
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
                    quote! {}
                }
            },
            syn::Data::Enum(en) => {
                let cases = map_variants(en.variants.iter(), |variant| {
                    let variant_name = &variant.ident;
                    match &variant.fields {
                        syn::Fields::Named(named) => {
                            let (field_indices, field_names): (Vec<syn::Index>, Vec<syn::Ident>) =
                                map_fields(named.named.iter(), |index, field| {
                                    (
                                        syn::Index::from(index),
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
                                #type_name::#variant_name { #(#field_names: var#field_indices),* } =>
                                #desc_name::#variant_name { #(#field_names: var#field_indices.describe()),* }
                            }
                        }
                        syn::Fields::Unnamed(unnamed) => {
                            let field_indices = map_fields(unnamed.unnamed.iter(), |index, _| {
                                syn::Index::from(index)
                            });
                            quote! {
                                #type_name::#variant_name(#(var#field_indices),*) =>
                                #desc_name::#variant_name(#(var#field_indices.describe()),*)
                            }
                        }
                        syn::Fields::Unit => {
                            quote! {
                                #type_name::#variant_name => #desc_name::#variant_name
                            }
                        }
                    }
                });
                quote! {
                    match self {
                        #(#cases),*
                    }
                }
            }
            syn::Data::Union(_un) => {
                panic!("delta_derive::generate_match_on_data not implemented for unions")
            }
        }
    }

    fn generate_change_from_data(inputs: &Inputs) -> Self {
        let visibility = &inputs.visibility;
        let change_name = format_ident!("{}Change", &inputs.input.ident);
        let change = Self::generate_type_definition(
            &inputs.visibility,
            &change_name,
            &map_on_types_of_fields_over_data(&inputs.input.data, |ty| {
                syn::parse2(Self::assoc_type(ty, "Change"))
                    .expect(&format!("Failed to parse associated type for Desc"))
            }),
        );
        // let is_unchanged_body =
        //     Self::generate_is_unchanged_body(&inputs.input.ident, &change_name, &inputs.input.data);
        Self {
            ty: syn::parse2(quote!(#change_name)).expect("Failed to parse Change type name"),
            definition: quote! {
                #[derive(Clone, Debug, PartialEq)]
                #change
            },
            method_body: Self::generate_delta_body(
                &inputs.input.ident,
                &change_name,
                &inputs.input.data,
            ),
            impl_section: None
            // impl_section: Some(quote! {
            //     impl #change_name {
            //         #visibility fn is_unchanged(&self) -> bool {
            //             match self {
            //                 #(#is_unchanged_body),*
            //             }
            //         }
            //     }
            // }),
        }
    }

    fn generate_delta_body(
        type_name: &syn::Ident,
        desc_name: &syn::Ident,
        data: &syn::Data,
    ) -> TokenStream {
        match data {
            syn::Data::Struct(st) => match &st.fields {
                syn::Fields::Named(named) => {
                    let field_names = map_fields(named.named.iter(), |_, field| {
                        field
                            .ident
                            .as_ref()
                            .expect("Found unnamed field in named struct")
                            .clone()
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
                    quote! {}
                }
            },
            syn::Data::Enum(en) => {
                let cases = map_variants(en.variants.iter(), |variant| {
                    let variant_name = &variant.ident;
                    match &variant.fields {
                        syn::Fields::Named(named) => {
                            let (field_indices, field_names): (Vec<syn::Index>, Vec<syn::Ident>) =
                                map_fields(named.named.iter(), |index, field| {
                                    (
                                        syn::Index::from(index),
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
                                #type_name::#variant_name { #(#field_names: var#field_indices),* } =>
                                #desc_name::#variant_name { #(#field_names: var#field_indices.describe()),* }
                            }
                        }
                        syn::Fields::Unnamed(unnamed) => {
                            let field_indices = map_fields(unnamed.unnamed.iter(), |index, _| {
                                syn::Index::from(index)
                            });
                            quote! {
                                #type_name::#variant_name(#(var#field_indices),*) =>
                                #desc_name::#variant_name(#(var#field_indices.describe()),*)
                            }
                        }
                        syn::Fields::Unit => {
                            quote! {
                                #type_name::#variant_name => #desc_name::#variant_name
                            }
                        }
                    }
                });
                quote! {
                    match self {
                        #(#cases),*
                    }
                }
            }
            syn::Data::Union(_un) => {
                panic!("delta_derive::generate_match_on_data not implemented for unions")
            }
        }
    }

    /*
        fn generate_is_unchanged_body(
            type_name: &syn::Ident,
            desc_name: &syn::Ident,
            data: &syn::Data,
        ) -> TokenStream {
            match data {
                syn::Data::Struct(st) => match &st.fields {
                    syn::Fields::Named(named) => {
                        let field_names = map_fields(named.named.iter(), |_, field| {
                            field
                                .ident
                                .as_ref()
                                .expect("Found unnamed field in named struct")
                                .clone()
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
                        quote! {}
                    }
                },
                syn::Data::Enum(en) => {
                    let cases = map_variants(en.variants.iter(), |variant| {
                        let variant_name = &variant.ident;
                        match &variant.fields {
                            syn::Fields::Named(named) => {
                                let (field_indices, field_names): (Vec<syn::Index>, Vec<syn::Ident>) =
                                    map_fields(named.named.iter(), |index, field| {
                                        (
                                            syn::Index::from(index),
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
                                    #type_name::#variant_name { #(#field_names: var#field_indices),* } =>
                                    #desc_name::#variant_name { #(#field_names: var#field_indices.describe()),* }
                                }
                            }
                            syn::Fields::Unnamed(unnamed) => {
                                let field_indices = map_fields(unnamed.unnamed.iter(), |index, _| {
                                    syn::Index::from(index)
                                });
                                quote! {
                                    #type_name::#variant_name(#(var#field_indices),*) =>
                                    #desc_name::#variant_name(#(var#field_indices.describe()),*)
                                }
                            }
                            syn::Fields::Unit => {
                                quote! {
                                    #type_name::#variant_name => #desc_name::#variant_name
                                }
                            }
                        }
                    });
                    quote! {
                        match self {
                            #(#cases),*
                        }
                    }
                }
                syn::Data::Union(_un) => {
                    panic!("delta_derive::generate_match_on_data not implemented for unions")
                }
            }
        }
    */

    fn generate_type_definition(
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
                                .expect("Found unnamed field in named struct")
                                .clone();
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
                                    .expect("Found unnamed field in named struct")
                                    .clone();
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
                panic!("delta_derive::generate_type_definition not implemented for unions")
            }
        };
        quote! {
            // #[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
            #[derive(PartialEq, Debug)]
            #visibility #keyword #type_name#body
        }
    }
}

impl quote::ToTokens for Definition {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Definition {
            ty: _,
            definition,
            method_body: _,
            impl_section,
        } = self;
        *tokens = quote! {
            #definition
            #impl_section
        }
    }
}

struct Outputs {
    desc: Option<Definition>,
    change: Option<Definition>,
}

fn impl_delta(
    name: &syn::Ident,
    describe_type: &syn::Type,
    describe_body: &TokenStream,
    change_type: &syn::Type,
    change_body: &TokenStream,
) -> TokenStream {
    quote! {
        impl delta::Delta for #name {
            type Desc = #describe_type;
            fn describe(&self) -> Self::Desc {
                #describe_body
            }

            type Change = #change_type;
            fn delta(&self, other: &Self) -> delta::Changed<Self::Change> {
                #change_body
            }
        }
    }
}

fn unit_type() -> syn::Type {
    syn::Type::Tuple(syn::TypeTuple {
        paren_token: syn::token::Paren {
            span: Span::call_site(),
        },
        elems: syn::punctuated::Punctuated::new(),
    })
}

impl Outputs {
    fn generate(self, inputs: &Inputs) -> TokenStream {
        let Outputs { desc, change } = self;
        let impl_delta = impl_delta(
            &inputs.input.ident,
            desc.as_ref().map(|d| &d.ty).unwrap_or(&unit_type()),
            desc.as_ref().map(|d| &d.method_body).unwrap_or(&quote!()),
            change.as_ref().map(|c| &c.ty).unwrap_or(&unit_type()),
            change
                .as_ref()
                .map(|c| &c.method_body)
                .unwrap_or(&quote!(delta::Changed::Unchanged)),
        );
        quote! {
            #desc
            #change
            #impl_delta
        }
    }
}

impl<'a> Inputs<'a> {
    fn process_data(&self) -> Outputs {
        match &self.input.data {
            syn::Data::Struct(st) => match &st.fields {
                syn::Fields::Unit => self.process_unit_struct(),
                syn::Fields::Unnamed(unnamed) => match unnamed.unnamed.len() {
                    0 => self.process_unit_struct(),
                    1 => self.process_struct_single_field(&unnamed.unnamed[0]),
                    _ => self.process_struct_unnamed_fields(&unnamed.unnamed),
                },
                syn::Fields::Named(named) => match named.named.len() {
                    0 => self.process_unit_struct(),
                    1 => self.process_struct_single_field(&named.named[0]),
                    _ => self.process_struct_named_fields(&named.named),
                },
            },
            syn::Data::Enum(en) => match en.variants.len() {
                0 => self.process_unit_struct(),
                1 => self.process_enum_single_variant(&en.variants[0]),
                _ => self.process_enum(&en.variants),
            },
            syn::Data::Union(_st) => {
                panic!("Delta derivation not yet implemented for unions");
            }
        }
    }

    fn process_unit_struct(&self) -> Outputs {
        Outputs {
            desc: None,
            change: None,
        }
    }

    fn process_struct_single_field(&self, _field: &syn::Field) -> Outputs {
        Outputs {
            desc: Some(Definition::generate_desc_from_data(self)),
            change: None,
        }
    }

    fn process_struct_unnamed_fields(
        &self,
        _fields: impl IntoIterator<Item = &'a syn::Field>,
    ) -> Outputs {
        panic!("NYI")
    }

    fn process_struct_named_fields(
        &self,
        _fields: impl IntoIterator<Item = &'a syn::Field>,
    ) -> Outputs {
        panic!("NYI")
    }

    fn process_enum_single_variant(&self, _variant: &syn::Variant) -> Outputs {
        panic!("NYI")
    }

    fn process_enum(&self, _variants: impl IntoIterator<Item = &'a syn::Variant>) -> Outputs {
        panic!("NYI")
    }

    fn process_enum_unit_variant(&self) -> Outputs {
        panic!("NYI")
    }

    fn process_enum_unit_single_variant(&self) -> Outputs {
        panic!("NYI")
    }

    fn process_enum_variant_single_unnamed_field(&self) -> Outputs {
        panic!("NYI")
    }

    fn process_enum_variant_unnamed_fields(&self) -> Outputs {
        panic!("NYI")
    }

    fn process_enum_variant_single_named_field(&self) -> Outputs {
        panic!("NYI")
    }

    fn process_enum_variant_named_fields(&self) -> Outputs {
        panic!("NYI")
    }
}

fn map_on_types_of_fields_over_data(
    data: &syn::Data,
    f: impl Fn(&syn::Type) -> syn::Type + Copy,
) -> syn::Data {
    match data {
        syn::Data::Struct(st) => syn::Data::Struct(syn::DataStruct {
            fields: map_on_types_of_fields(&st.fields, f),
            ..st.clone()
        }),
        syn::Data::Enum(en) => syn::Data::Enum(syn::DataEnum {
            variants: FromIterator::from_iter(map_variants(&en.variants, move |v| syn::Variant {
                fields: map_on_types_of_fields(&v.fields, f),
                ..v.clone()
            })),
            ..en.clone()
        }),
        syn::Data::Union(un) => syn::Data::Union(syn::DataUnion {
            fields: syn::FieldsNamed {
                named: FromIterator::from_iter(map_field_types(un.fields.named.iter(), f)),
                ..un.fields.clone()
            },
            ..un.clone()
        }),
    }
}

fn map_on_types_of_fields(
    fields: &syn::Fields,
    f: impl Fn(&syn::Type) -> syn::Type,
) -> syn::Fields {
    match fields {
        syn::Fields::Named(named) => syn::Fields::Named(syn::FieldsNamed {
            named: FromIterator::from_iter(map_field_types(named.named.iter(), f)),
            ..named.clone()
        }),
        syn::Fields::Unnamed(unnamed) => syn::Fields::Unnamed(syn::FieldsUnnamed {
            unnamed: FromIterator::from_iter(map_field_types(unnamed.unnamed.iter(), f)),
            ..unnamed.clone()
        }),
        syn::Fields::Unit => syn::Fields::Unit,
    }
}

fn map_field_types<'a>(
    fields: impl IntoIterator<Item = &'a syn::Field>,
    f: impl Fn(&'a syn::Type) -> syn::Type,
) -> Vec<syn::Field> {
    fields
        .into_iter()
        .map(|field| {
            if has_attr(&field.attrs, "delta_ignore").is_none() {
                Some(syn::Field {
                    ty: f(&field.ty),
                    ..field.clone()
                })
            } else {
                None
            }
        })
        .flatten()
        .collect()
}

/*
fn map_over_fields<R>(fields: &syn::Fields, f: impl Fn(usize, &syn::Field) -> R) -> Vec<R> {
    match fields {
        syn::Fields::Named(named) => map_fields(named.named.iter(), f),
        syn::Fields::Unnamed(unnamed) => map_fields(unnamed.unnamed.iter(), f),
        syn::Fields::Unit => Vec::new(),
    }
}
*/

fn map_fields<'a, R>(
    fields: impl IntoIterator<Item = &'a syn::Field>,
    f: impl Fn(usize, &syn::Field) -> R,
) -> Vec<R> {
    let mut result = Vec::<R>::new();
    for (field, index) in fields.into_iter().zip(0usize..) {
        if has_attr(&field.attrs, "delta_ignore").is_none() {
            result.push(f(index, &field));
        }
    }
    result
}

fn map_variants<'a, R>(
    variants: impl IntoIterator<Item = &'a syn::Variant>,
    f: impl Fn(&syn::Variant) -> R,
) -> Vec<R> {
    let mut result = Vec::<R>::new();
    for variant in variants {
        if has_attr(&variant.attrs, "delta_ignore").is_none() {
            result.push(f(&variant));
        }
    }
    result
}

/*
fn process_struct(attrs: &Attributes, st: &syn::DataStruct) -> TokenStream {
    let name_and_types = field_names_and_types(&st.fields);
    if name_and_types.is_empty() {
        let delta_impl = define_delta_impl(
            type_name,
            desc_type,
            desc_body,
            &quote!(()),
            &quote!(delta::Changed::Unchanged),
        );

        let gen = quote! {
            #delta_impl
        };
        gen.into()
    } else if name_and_types.len() == 1 {
        let FieldInfo {
            name: field_name,
            pascal_case: _,
            ty,
        } = &name_and_types[0];
        let ch = change_type(ty);
        let change_innards = vec![quote!(#ch)];
        let change_struct = definition(
            visibility,
            quote!(struct),
            change_name,
            false,
            change_innards,
        );
        let delta_impl = define_delta_impl(
            type_name,
            desc_type,
            desc_body,
            &quote!(#change_name),
            &quote! {
                self.#field_name.delta(&other.#field_name).map(#change_name)
            },
        );

        let gen = quote! {
            #change_struct
            #delta_impl
        };
        gen.into()
    } else {
        let change_struct = define_enum_from_fields(visibility, change_name, &st.fields);

        let delta_innards: Vec<TokenStream> =
            name_and_types.iter().map(
                |FieldInfo {
                   name,
                   pascal_case,
                   ty: _,
                }|
                {
                    quote!(self.#name.delta(&other.#name).map(#change_name::#pascal_case).to_changes())
                }).collect();
        let delta_impl = define_delta_impl(
            type_name,
            desc_type,
            desc_body,
            &quote!(Vec<#change_name>),
            &quote! {
                let changes: Vec<#change_name> = vec![
                    #(#delta_innards),*
                ]
                    .into_iter()
                    .flatten()
                    .collect();
                if changes.is_empty() {
                    delta::Changed::Unchanged
                } else {
                    delta::Changed::Changed(changes)
                }
            },
        );

        let gen = quote! {
            #change_struct
            #delta_impl
        };
        gen.into()
    }
}

#[allow(clippy::cognitive_complexity)]
fn process_enum(attrs: &Attributes, en: &syn::DataEnum) -> TokenStream {
    let mut desc_innards = Vec::<TokenStream>::new();
    let mut match_innards = Vec::<TokenStream>::new();
    let mut change_innards = Vec::<TokenStream>::new();
    let mut is_unchanged_innards = Vec::<TokenStream>::new();
    let mut delta_innards = Vec::<TokenStream>::new();

    for variant in en.variants.iter() {
        // jww (2021-10-30): Also need to check for delta_ignore on the
        // variant's fields.
        if has_attr(&variant.attrs, "delta_ignore").is_none() {
            let vname = &variant.ident;

            // jww (2021-10-30): This is what needs to happen, rather than the
            // complicated code below: Using the name of the original struct
            // (Foo), the name of the variant (Bar), and the set of fields for
            // that variant, define a structure named `FooBar` that gives a
            // concrete type for that variant's fields. Then the Change for
            // that variant is Bar(<FooBar as Delta>::Change), after deriving
            // Delta for the generated struct.

            let _fields_change_struct = create_mirror_struct(
                visibility,
                &format_ident!("{}{}", type_name, vname),
                &"Change",
                &variant.fields,
                false,
            );

            match &variant.fields {
                Fields::Named(named) => {
                    let desc_decls: Vec<TokenStream> = named
                        .named
                        .iter()
                        .map(|field| {
                            let ident = &field.ident;
                            let ty = desc_type(&field.ty);
                            quote!(#ident: #ty)
                        })
                        .collect();
                    desc_innards.push(quote!(#vname { #(#desc_decls),* }));

                    let field_decls: Vec<TokenStream> = named
                        .named
                        .iter()
                        .map(|field| {
                            let ident = &field.ident;
                            let ty = change_type(&field.ty);
                            quote!(#ident: delta::Changed<#ty>)
                        })
                        .collect();
                    change_innards.push(quote!(#vname { #(#field_decls),* }));

                    let idents: Vec<&syn::Ident> = named
                        .named
                        .iter()
                        .map(|field| field.ident.as_ref().unwrap())
                        .collect();
                    let vars: Box<dyn Fn(&str) -> Vec<TokenStream>> =
                        Box::new(|prefix| {
                            named
                                .named
                                .iter()
                                .zip(0usize..)
                                .map(|(_field, index)| {
                                    let var = format_ident!("{}_var{}", prefix, index);
                                    quote!(#var)
                                })
                                .collect()
                        });
                    let self_vars = vars("self");
                    let other_vars = vars("other");

                    match_innards.push(quote! {
                        #type_name::#vname { #(#idents: #self_vars),* } =>
                            #desc_name::#vname {
                                #(#idents: #self_vars.describe()),*
                            }
                    });

                    is_unchanged_innards.push(quote! {
                        #change_name::#vname { #(#idents: #self_vars),* } =>
                            vec![#(#self_vars.is_unchanged()),*].into_iter().all(std::convert::identity)
                    });

                    delta_innards.push(quote! {
                        (#type_name::#vname { #(#idents: #self_vars),* },
                         #type_name::#vname { #(#idents: #other_vars),* }) => {
                            let change = #change_name::#vname {
                                #(#idents: #self_vars.delta(&#other_vars)),*
                            };
                            if change.is_unchanged() {
                                delta::Changed::Unchanged
                            } else {
                                delta::Changed::Changed(delta::EnumChange::SameVariant(change))
                            }
                        }
                    });
                }
                Fields::Unnamed(unnamed) => {
                    let desc_decls: Vec<TokenStream> = unnamed
                        .unnamed
                        .iter()
                        .map(|field| {
                            let ty = desc_type(&field.ty);
                            quote!(#ty)
                        })
                        .collect();
                    desc_innards.push(quote!(#vname(#(#desc_decls),*)));

                    let field_decls: Vec<TokenStream> = unnamed
                        .unnamed
                        .iter()
                        .map(|field| {
                            let ty = change_type(&field.ty);
                            quote!(delta::Changed<#ty>)
                        })
                        .collect();
                    change_innards.push(quote!(#vname(#(#field_decls),*)));

                    let vars: Box<dyn Fn(&str) -> Vec<TokenStream>> =
                        Box::new(|prefix| {
                            unnamed
                                .unnamed
                                .iter()
                                .zip(0usize..)
                                .map(|(_field, index)| {
                                    let var: syn::Ident = Ident::new(
                                        &format!("{}_var{}", prefix, index),
                                        Span::call_site(),
                                    );
                                    quote!(#var)
                                })
                                .collect()
                        });
                    let self_vars = vars("self");
                    let other_vars = vars("other");

                    match_innards.push(quote! {
                        #type_name::#vname(#(#self_vars),*) =>
                            #desc_name::#vname(#(#self_vars.describe()),*)
                    });

                    is_unchanged_innards.push(quote! {
                        #change_name::#vname(#(#self_vars),*) =>
                            vec![#(#self_vars.is_unchanged()),*].into_iter().all(std::convert::identity)
                    });

                    delta_innards.push(quote! {
                        (#type_name::#vname(#(#self_vars),*),
                         #type_name::#vname(#(#other_vars),*)) => {
                            let change = #change_name::#vname(#(#self_vars.delta(&#other_vars)),*);
                            if change.is_unchanged() {
                                delta::Changed::Unchanged
                            } else {
                                delta::Changed::Changed(delta::EnumChange::SameVariant(change))
                            }
                        }
                    });
                }
                Fields::Unit => {
                    desc_innards.push(quote!(#vname));
                    change_innards.push(quote!(#vname));
                    match_innards.push(quote!(#type_name::#vname => #desc_name::#vname));
                    is_unchanged_innards.push(quote!(
                        #change_name::#vname => true
                    ));
                    delta_innards.push(
                        quote!((#type_name::#vname, #type_name::#vname) => delta::Changed::Unchanged),
                    );
                }
            }
        }
    }

    delta_innards.push(quote! {
        (_, _) => delta::Changed::Changed(
            delta::EnumChange::DiffVariant(
                self.describe(), other.describe()))
    });

    let desc_struct = definition(visibility, quote!(enum), desc_name, false, desc_innards);
    let change_struct = definition(visibility, quote!(enum), change_name, false, change_innards);

    let delta_impl = define_delta_impl(
        type_name,
        &quote!(#desc_name),
        &quote! {
            match self {
                #(#match_innards),*
            }
        },
        &quote!(delta::EnumChange<Self::Desc, #change_name>),
        &quote! {
            match (self, other) {
                #(#delta_innards),*
            }
        },
    );

    let gen = quote! {
        #desc_struct
        #change_struct

        impl #change_name {
            #visibility fn is_unchanged(&self) -> bool {
                match self {
                    #(#is_unchanged_innards),*
                }
            }
        }

        #delta_impl
    };
    gen.into()
}

struct FieldInfo<'a> {
    name: Box<dyn ToTokens>,
    pascal_case: syn::Ident,
    ty: &'a syn::Type,
}

fn field_names_and_types(fields: &syn::Fields) -> Vec<FieldInfo> {
    let mut result = Vec::new();
    match fields {
        Fields::Named(named) => {
            for field in named.named.iter() {
                if has_attr(&field.attrs, "delta_ignore").is_none() {
                    let name: &syn::Ident = field.ident.as_ref().unwrap();
                    let capitalized: syn::Ident =
                        Ident::new(&name.to_string().to_case(Case::Pascal), Span::call_site());
                    result.push(FieldInfo {
                        name: Box::new(name.clone()),
                        pascal_case: capitalized,
                        ty: &field.ty,
                    });
                }
            }
        }
        Fields::Unnamed(unnamed) => {
            for (field, index) in unnamed.unnamed.iter().zip(0usize..) {
                if has_attr(&field.attrs, "delta_ignore").is_none() {
                    let name: syn::Index = Index::from(index);
                    let capitalized: syn::Ident = format_ident!("Field{}", index);
                    result.push(FieldInfo {
                        name: Box::new(name),
                        pascal_case: capitalized,
                        ty: &field.ty,
                    });
                }
            }
        }
        Fields::Unit => {}
    }
    result
}

fn _create_data_struct(fields: &syn::Fields) -> syn::DataStruct {
    syn::DataStruct {
        struct_token: syn::token::Struct {
            span: Span::call_site(),
        },
        fields: fields.clone(),
        semi_token: None,
    }
}

/// A mirror struct copies the exact fields of another structure (unless it
/// had unnamed fields, and `use_unnamed_fields` is false, in which case all
/// the unnamed fields will be given names of field0, field1, etc.). During
/// the copy, however, the types are substituted by an associated type of the
/// `Delta` trait.
fn create_mirror_struct(
    visibility: &syn::Visibility,
    type_name: &syn::Ident,
    suffix: &str,
    fields: &syn::Fields,
    use_unnamed_fields: bool,
) -> TokenStream {
    define_struct_from_fields(
        visibility,
        &format_ident!("{}{}", type_name, suffix),
        #[allow(unused_variables)] // compiler doesn't see the use of ty
        &map_field_types(&fields, |ty: &syn::Type| -> syn::Type {
            let suffix_ident = format_ident!("{}", suffix);
            syn::parse2(quote!(<#ty as delta::Delta>::#suffix_ident))
                .expect(&format!("Failed to parse associated type for {}", suffix))
        }),
        use_unnamed_fields,
    )
}

fn define_enum_from_fields(
    visibility: &syn::Visibility,
    name: &syn::Ident,
    fields: &syn::Fields,
) -> TokenStream {
    let change_innards: Vec<TokenStream> = field_names_and_types(fields)
        .iter()
        .map(
            |FieldInfo {
                 name: _,
                 pascal_case,
                 ty,
             }| {
                let ch = change_type(ty);
                quote!(#pascal_case(#ch))
            },
        )
        .collect();
    definition(visibility, quote!(enum), name, false, change_innards)
}

fn define_struct_from_fields(
    visibility: &syn::Visibility,
    name: &syn::Ident,
    fields: &syn::Fields,
    use_unnamed_fields: bool,
) -> TokenStream {
    let mut struct_fields = Vec::<TokenStream>::new();
    match &fields {
        Fields::Named(named) => {
            for field in named.named.iter() {
                if has_attr(&field.attrs, "delta_ignore").is_none() {
                    let field_name: &syn::Ident = field.ident.as_ref().unwrap();
                    let ty = &field.ty;
                    struct_fields.push(quote!(#field_name: #ty));
                }
            }
        }
        Fields::Unnamed(unnamed) => {
            for (field, index) in unnamed.unnamed.iter().zip(0usize..) {
                if has_attr(&field.attrs, "delta_ignore").is_none() {
                    let ty = &field.ty;
                    if use_unnamed_fields {
                        struct_fields.push(quote!(#ty));
                    } else {
                        let field_name: syn::Ident =
                            Ident::new(&format!("field{}", index), Span::call_site());
                        struct_fields.push(quote!(#field_name: #ty));
                    }
                }
            }
        }
        Fields::Unit => {}
    }
    definition(
        visibility,
        quote!(struct),
        name,
        use_unnamed_fields,
        struct_fields,
    )
}
*/
