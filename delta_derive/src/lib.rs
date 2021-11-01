use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use std::iter::FromIterator;

fn has_attr<'a>(attrs: &'a [syn::Attribute], attr_name: &str) -> Option<&'a syn::Attribute> {
    attrs.iter().find(|attr| attr.path.is_ident(attr_name))
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
    definition: Option<TokenStream>,
    // For `Desc` types, the method body is for `describe`.
    // For `Change` types, the method body is for `delta`.
    method_body: TokenStream,
}

impl Definition {
    fn ident_to_type(ident: &syn::Ident) -> syn::Type {
        syn::parse2(quote!(#ident)).unwrap_or_else(|_| panic!("Failed to parse type"))
    }

    fn assoc_type(ty: &syn::Type, name: &str) -> syn::Type {
        let ident = format_ident!("{}", name);
        syn::parse2(quote!(<#ty as delta::Delta>::#ident))
            .unwrap_or_else(|_| panic!("Failed to parse associated type"))
    }

    fn changed_type(ty: &syn::Type) -> syn::Type {
        syn::parse2(quote!(delta::Changed<#ty>))
            .unwrap_or_else(|_| panic!("Failed to parse Changed type"))
    }

    fn generate_desc_from_data(inputs: &Inputs) -> Self {
        let type_name = &inputs.input.ident;
        let desc_name = format_ident!("{}Desc", &inputs.input.ident);
        let desc_type = Self::generate_type_definition(
            &inputs.visibility,
            &desc_name,
            &map_on_fields_over_data(&inputs.input.data, |_, field| syn::Field {
                ty: Self::assoc_type(&field.ty, "Desc"),
                ..field.clone()
            }),
        );
        Self {
            ty: inputs
                .attrs
                .describe_type
                .as_ref()
                .unwrap_or(
                    &syn::parse2(if inputs.attrs.compare_default {
                        quote!(Self::Change)
                    } else {
                        quote!(#desc_name)
                    })
                    .expect("Failed to parse Desc type name"),
                )
                .clone(),
            definition: if inputs.attrs.describe_type.is_some() || inputs.attrs.compare_default {
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
                .unwrap_or(if inputs.attrs.compare_default {
                    quote!(#type_name::default().delta(self).unwrap_or_default())
                } else {
                    Self::generate_describe_body(
                        &inputs.input.ident,
                        &desc_name,
                        &inputs.input.data,
                    )
                }),
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
            },
            syn::Data::Enum(en) => {
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
            syn::Data::Union(_un) => {
                panic!("delta_derive::generate_describe_body not implemented for unions")
            }
        }
    }

    fn generate_change_from_data(inputs: &Inputs) -> Self {
        let type_name = &inputs.input.ident;
        let change_name = format_ident!("{}Change", type_name);
        let change_type = Self::generate_type_definition(
            &inputs.visibility,
            &change_name,
            &Self::generate_change_type(&inputs.input.ident, &inputs.input.data),
        );
        Self {
            ty: syn::parse2(if Self::is_struct_with_many_fields(&inputs.input.data) {
                quote!(Vec<#change_name>)
            } else {
                quote!(#change_name)
            })
            .expect("Failed to parse Change type name"),
            definition: Some(quote!(#change_type)),
            method_body: Self::generate_delta_body(type_name, &change_name, &inputs.input.data),
        }
    }

    fn generate_change_type_from_datastruct(st: &syn::DataStruct) -> syn::Data {
        if Self::is_datastruct_with_many_fields(st) {
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
            // A singleton struct is handled differently, since the
            // only change that could occur is in the single field, we
            // only need to store that change data, rather than the
            // varying combinations that could occur in the case of
            // multiple fields.
            map_on_fields_over_datastruct(st, |_, field| syn::Field {
                ty: Self::assoc_type(&field.ty, "Change"),
                ..field.clone()
            })
        }
    }

    fn generate_change_type_from_dataenum(type_name: &syn::Ident, en: &syn::DataEnum) -> syn::Data {
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
                                        let change_type = Self::assoc_type(&field.ty, "Change");
                                        if many_fields {
                                            Self::changed_type(&change_type)
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
                                ty: Self::assoc_type(&Self::ident_to_type(type_name), "Desc"),
                                attrs: Default::default(),
                                vis: syn::Visibility::Inherited,
                                colon_token: Default::default(),
                            };
                            syn::FieldsUnnamed {
                                unnamed: FromIterator::from_iter(vec![
                                    desc_field.clone(),
                                    desc_field,
                                ]),
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

    fn generate_change_type(type_name: &syn::Ident, data: &syn::Data) -> syn::Data {
        match data {
            syn::Data::Struct(st) => Self::generate_change_type_from_datastruct(st),
            syn::Data::Enum(en) => Self::generate_change_type_from_dataenum(type_name, en),
            syn::Data::Union(_un) => {
                panic!("delta_derive::generate_change_type not implemented for unions")
            }
        }
    }

    #[allow(clippy::cognitive_complexity)]
    fn generate_delta_body(
        type_name: &syn::Ident,
        change_name: &syn::Ident,
        data: &syn::Data,
    ) -> TokenStream {
        match data {
            syn::Data::Struct(st) => {
                let inspect_field =
                    |index: usize, field: &syn::Field| -> (TokenStream, syn::Ident) {
                        if let Some(name) = field.ident.as_ref() {
                            (
                                quote!(#name),
                                syn::Ident::new(
                                    &name.to_string().to_case(Case::Pascal),
                                    Span::call_site(),
                                ),
                            )
                        } else {
                            (
                                {
                                    let idx = syn::Index::from(index);
                                    quote!(#idx)
                                },
                                format_ident!("Field{}", index),
                            )
                        }
                    };
                let (field_names, field_variants): (Vec<TokenStream>, Vec<syn::Ident>) =
                    match &st.fields {
                        syn::Fields::Named(named) => map_fields(named.named.iter(), inspect_field)
                            .into_iter()
                            .unzip(),
                        syn::Fields::Unnamed(unnamed) => {
                            map_fields(unnamed.unnamed.iter(), inspect_field)
                                .into_iter()
                                .unzip()
                        }
                        syn::Fields::Unit => (Vec::new(), Vec::new()),
                    };
                if Self::is_datastruct_with_many_fields(st) {
                    quote! {
                        let changes: Vec<#change_name> = vec![
                            #(self.#field_names.delta(&other.#field_names)
                                  .map(#change_name::#field_variants)),*
                        ]
                            .into_iter()
                            .flatten()
                            .collect();
                        if changes.is_empty() {
                            delta::Changed::Unchanged
                        } else {
                            delta::Changed::Changed(changes)
                        }
                    }
                } else {
                    quote! {
                        #(self.#field_names.delta(&other.#field_names).map(#change_name))*
                    }
                }
            }
            syn::Data::Enum(en) => {
                if en.variants.len() < 1 {
                    quote! {
                        delta::Changed::Unchanged
                    }
                } else {
                    let inspect_variant = |prefix: &syn::Ident,
                                           variant: &syn::Variant|
                     -> (TokenStream, TokenStream) {
                        match &variant.fields {
                            syn::Fields::Named(named) => {
                                let (vars_and_changes, fields): (
                                    Vec<(syn::Ident, syn::Ident)>,
                                    Vec<&syn::Ident>,
                                ) = map_fields(named.named.iter(), |index, field| {
                                    (
                                        (
                                            format_ident!("{}_var{}", prefix, index),
                                            format_ident!("changes_var{}", index),
                                        ),
                                        field
                                            .ident
                                            .as_ref()
                                            .expect("Found unnamed field in named struct"),
                                    )
                                })
                                .into_iter()
                                .unzip();
                                let (vars, changes): (Vec<syn::Ident>, Vec<syn::Ident>) =
                                    vars_and_changes.into_iter().unzip();
                                (
                                    quote!({ #(#fields: #vars),* }),
                                    quote!({ #(#fields: #changes),* }),
                                )
                            }
                            syn::Fields::Unnamed(unnamed) => {
                                let (vars, changes): (Vec<syn::Ident>, Vec<syn::Ident>) =
                                    map_fields(unnamed.unnamed.iter(), |index, _| {
                                        (
                                            format_ident!("{}_var{}", prefix, index),
                                            format_ident!("changes_var{}", index),
                                        )
                                    })
                                    .into_iter()
                                    .unzip();
                                (quote!((#(#vars),*)), quote!((#(#changes),*)))
                            }
                            syn::Fields::Unit => (quote! {}, quote! {}),
                        }
                    };
                    let variant_captures = |prefix: &syn::Ident| -> Vec<TokenStream> {
                        map_variants(en.variants.iter(), |v| inspect_variant(prefix, v).0)
                    };
                    let variant_captures_self = variant_captures(&format_ident!("self"));
                    let variant_captures_other = variant_captures(&format_ident!("other"));

                    let (let_vars, return_results): (Vec<TokenStream>, Vec<TokenStream>) =
                        map_variants(en.variants.iter(), |variant| {
                            let variant_name = &variant.ident;
                            let (_, assignments) =
                                inspect_variant(&format_ident!("not_used"), variant);
                            let (changes_vars, delta_calls): (Vec<syn::Ident>, Vec<TokenStream>) =
                                map_over_fields(&variant.fields, |index, _| {
                                    (format_ident!("changes_var{}", index), {
                                        let self_var = format_ident!("self_var{}", index);
                                        let other_var = format_ident!("other_var{}", index);
                                        quote! {
                                            #self_var.delta(&#other_var)
                                        }
                                    })
                                })
                                .into_iter()
                                .unzip();
                            let both_ident = format_ident!("Both{}", variant_name);
                            (
                                quote! {
                                    #(let #changes_vars = #delta_calls;)*
                                },
                                if changes_vars.is_empty() {
                                    quote!(delta::Changed::Unchanged)
                                } else if changes_vars.len() == 1 {
                                    quote! {
                                        #(#changes_vars.map(
                                            |changes_var0|
                                            #change_name::#both_ident #assignments))*
                                    }
                                } else {
                                    quote! {
                                        if #(#changes_vars.is_unchanged())&&* {
                                            delta::Changed::Unchanged
                                        } else {
                                            delta::Changed::Changed(
                                                #change_name::#both_ident #assignments
                                            )
                                        }
                                    }
                                },
                            )
                        })
                        .into_iter()
                        .unzip();

                    let variant_names: Vec<syn::Ident> =
                        map_variants(en.variants.iter(), |variant| variant.ident.clone())
                            .into_iter()
                            .collect();

                    let default_case = if en.variants.len() > 1 {
                        quote! {
                            (_, _) => delta::Changed::Changed(
                                #change_name::Different(self.describe(), other.describe()))
                        }
                    } else {
                        quote!()
                    };

                    quote! {
                        match (self, other) {
                            #((#type_name::#variant_names #variant_captures_self,
                               #type_name::#variant_names #variant_captures_other) => {
                              #let_vars
                              #return_results
                            }),*
                            #default_case
                        }
                    }
                }
            }
            syn::Data::Union(_un) => {
                panic!("delta_derive::generate_delta_body not implemented for unions")
            }
        }
    }

    fn is_datastruct_with_many_fields(st: &syn::DataStruct) -> bool {
        match &st.fields {
            syn::Fields::Named(named) => named.named.len() > 1,
            syn::Fields::Unnamed(unnamed) => unnamed.unnamed.len() > 1,
            syn::Fields::Unit => false,
        }
    }

    fn is_struct_with_many_fields(data: &syn::Data) -> bool {
        match data {
            syn::Data::Struct(st) => Self::is_datastruct_with_many_fields(st),
            _ => false,
        }
    }

    fn _variant_to_datastruct(variant: &syn::Variant) -> syn::DataStruct {
        syn::DataStruct {
            fields: variant.fields.clone(),
            struct_token: Default::default(),
            semi_token: Default::default(),
        }
    }

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
        if let Some(body) = &self.definition {
            *tokens = quote!(#body);
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
        #[allow(unused_variables)] // compiler doesn't see the use of x
        let desc = desc.map(|x| quote!(#x)).unwrap_or_default();
        #[allow(unused_variables)] // compiler doesn't see the use of x
        let change = change.map(|x| quote!(#x)).unwrap_or_default();
        quote! {
            #desc
            #change
            #impl_delta
        }
    }
}

impl<'a> Inputs<'a> {
    fn process_data(&self) -> Outputs {
        let is_unitary = match &self.input.data {
            syn::Data::Struct(st) => match &st.fields {
                syn::Fields::Unit => true,
                syn::Fields::Unnamed(unnamed) => unnamed.unnamed.is_empty(),
                syn::Fields::Named(named) => named.named.is_empty(),
            },
            syn::Data::Enum(en) => en.variants.is_empty(),
            syn::Data::Union(_st) => {
                panic!("Delta derivation not available for unions");
            }
        };
        self.process_struct_or_enum(is_unitary)
    }

    fn process_struct_or_enum(&self, is_unitary: bool) -> Outputs {
        Outputs {
            desc: if self.attrs.no_description {
                None
            } else {
                Some(Definition::generate_desc_from_data(self))
            },
            change: if is_unitary {
                None
            } else {
                Some(Definition::generate_change_from_data(self))
            },
        }
    }
}

fn map_on_fields_over_data(
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

fn map_on_fields_over_datastruct(
    st: &syn::DataStruct,
    f: impl Fn(usize, &syn::Field) -> syn::Field,
) -> syn::Data {
    syn::Data::Struct(syn::DataStruct {
        fields: map_on_fields(&st.fields, f),
        ..*st
    })
}

fn _map_on_variants_over_dataenum(
    en: &syn::DataEnum,
    f: impl Fn(&syn::Variant) -> syn::Variant,
) -> syn::Data {
    syn::Data::Enum(syn::DataEnum {
        variants: FromIterator::from_iter(map_variants(en.variants.iter(), f)),
        ..*en
    })
}

fn map_on_fields(
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

fn map_over_fields<R>(fields: &syn::Fields, f: impl Fn(usize, &syn::Field) -> R) -> Vec<R> {
    match fields {
        syn::Fields::Named(named) => map_fields(named.named.iter(), f),
        syn::Fields::Unnamed(unnamed) => map_fields(unnamed.unnamed.iter(), f),
        syn::Fields::Unit => Vec::new(),
    }
}

fn map_fields<'a, R>(
    fields: impl IntoIterator<Item = &'a syn::Field>,
    f: impl Fn(usize, &'a syn::Field) -> R,
) -> Vec<R> {
    fields
        .into_iter()
        .zip(0usize..)
        .map(|(field, index)| {
            if has_attr(&field.attrs, "delta_ignore").is_none() {
                Some(f(index, field))
            } else {
                None
            }
        })
        .flatten()
        .collect()
}

fn map_variants<'a, R>(
    variants: impl IntoIterator<Item = &'a syn::Variant>,
    f: impl Fn(&syn::Variant) -> R,
) -> Vec<R> {
    variants
        .into_iter()
        .map(|variant| {
            if has_attr(&variant.attrs, "delta_ignore").is_none() {
                Some(f(variant))
            } else {
                None
            }
        })
        .flatten()
        .collect()
}
