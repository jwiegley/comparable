use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident, Index};

#[proc_macro_derive(
    Delta,
    attributes(
        describe_type,
        describe_body,
        no_description,
        compare_default,
        public_change,
        private_change,
        delta_ignore
    )
)]
pub fn delta_macro(input: TokenStream) -> TokenStream {
    impl_delta(parse_macro_input!(input as DeriveInput))
}

#[allow(clippy::cognitive_complexity)]
fn impl_delta(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let has_attr = |attrs: &Vec<syn::Attribute>, attr_name| -> Option<syn::Attribute> {
        attrs
            .iter()
            .filter(|attr| attr.path.is_ident(attr_name))
            .peekable()
            .peek()
            .map(|x| (*x).clone())
    };
    let compare_default = has_attr(&input.attrs, "compare_default").is_some();
    let visibility = if has_attr(&input.attrs, "public_change").is_some() {
        syn::Visibility::Public(syn::VisPublic {
            pub_token: syn::token::Pub {
                span: Span::call_site(),
            },
        })
    } else if has_attr(&input.attrs, "private_change").is_some() {
        syn::Visibility::Inherited
    } else {
        input.vis
    };
    let change = format_ident!("{}Change", name);
    let describe_type = if has_attr(&input.attrs, "no_description").is_some() {
        quote!(())
    } else if compare_default {
        quote!(Self::Change)
    } else {
        quote!(Self)
    };
    let describe_body = if has_attr(&input.attrs, "no_description").is_some() {
        quote!(())
    } else if compare_default {
        quote!(#name::default().delta(self).unwrap_or_default())
    } else {
        quote!((*self).clone())
    };
    let representation = has_attr(&input.attrs, "describe_type")
        .map(|x| {
            x.parse_args::<syn::Type>()
                .expect("Failed to parse \"describe_type\" attribute")
                .into_token_stream()
        })
        .unwrap_or(describe_type);
    let description = has_attr(&input.attrs, "describe_body")
        .map(|x| {
            x.parse_args::<syn::Expr>()
                .expect("Failed to parse \"describe_body\" attribute")
                .into_token_stream()
        })
        .unwrap_or(describe_body);
    match &input.data {
        Data::Struct(st) => {
            let mut field_names_and_types: Vec<(Box<dyn ToTokens>, syn::Ident, &syn::Type)> =
                Vec::new();
            match &st.fields {
                Fields::Named(named) => {
                    for field in named.named.iter() {
                        if has_attr(&field.attrs, "delta_ignore").is_none() {
                            let name: &syn::Ident = field.ident.as_ref().unwrap();
                            let capitalized: syn::Ident = Ident::new(
                                &name.to_string().to_case(Case::Pascal),
                                Span::call_site(),
                            );
                            field_names_and_types.push((
                                Box::new(name.clone()),
                                capitalized,
                                &field.ty,
                            ));
                        }
                    }
                }
                Fields::Unnamed(unnamed) => {
                    for (field, index) in unnamed.unnamed.iter().zip(0usize..) {
                        if has_attr(&field.attrs, "delta_ignore").is_none() {
                            let name: syn::Index = Index::from(index);
                            let capitalized: syn::Ident =
                                Ident::new(&format!("Field{}", index), Span::call_site());
                            field_names_and_types.push((Box::new(name), capitalized, &field.ty));
                        }
                    }
                }
                Fields::Unit => {}
            }

            let mut change_innards = Vec::new();
            let mut delta_innards = Vec::new();

            for (name, capitalized, typename) in field_names_and_types.iter() {
                change_innards.push(quote!(#capitalized(<#typename as delta::Delta>::Change)));
                delta_innards.push(
                    quote!(self.#name.delta(&other.#name).map(#change::#capitalized).to_changes()),
                );
            }

            let gen = if change_innards.is_empty() {
                quote! {
                // #[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
                #[derive(PartialEq, Debug)]
                #visibility struct #change;

                impl delta::Delta for #name {
                    type Desc = #representation;

                    fn describe(&self) -> Self::Desc {
                        #description
                    }

                    type Change = #change;

                    fn delta(&self, _other: &Self) -> delta::Changed<Self::Change> {
                        delta::Changed::Unchanged
                    }
                }
                }
            } else {
                quote! {
                // #[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
                #[derive(PartialEq, Debug)]
                #visibility enum #change {
                    #(#change_innards),*
                }

                impl delta::Delta for #name {
                    type Desc = #representation;

                    fn describe(&self) -> Self::Desc {
                        #description
                    }

                    type Change = Vec<#change>;

                    fn delta(&self, other: &Self) -> delta::Changed<Self::Change> {
                        let changes: Vec<#change> = vec![
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
                    }
                }
                }
            };
            gen.into()
        }
        Data::Enum(en) => {
            let desc = format_ident!("{}Desc", name);

            let mut desc_innards: Vec<proc_macro2::TokenStream> = Vec::new();
            let mut match_innards: Vec<proc_macro2::TokenStream> = Vec::new();
            let mut change_innards: Vec<proc_macro2::TokenStream> = Vec::new();
            let mut delta_innards: Vec<proc_macro2::TokenStream> = Vec::new();

            for variant in en.variants.iter() {
                if has_attr(&variant.attrs, "delta_ignore").is_none() {
                    let vname = &variant.ident;
                    match &variant.fields {
                        Fields::Named(named) => {
                            let desc_decls: Vec<proc_macro2::TokenStream> = named
                                .named
                                .iter()
                                .map(|field| {
                                    let ident = &field.ident;
                                    let ty = &field.ty;
                                    quote!(#ident: <#ty as delta::Delta>::Desc)
                                })
                                .collect();
                            desc_innards.push(quote!(#vname { #(#desc_decls),* }));

                            let field_decls: Vec<proc_macro2::TokenStream> = named
                                .named
                                .iter()
                                .map(|field| {
                                    let ident = &field.ident;
                                    let ty = &field.ty;
                                    quote!(#ident: delta::Changed<<#ty as delta::Delta>::Change>)
                                })
                                .collect();
                            change_innards.push(quote!(#vname { #(#field_decls),* }));

                            let idents: Vec<&syn::Ident> = named
                                .named
                                .iter()
                                .map(|field| field.ident.as_ref().unwrap())
                                .collect();
                            let vars: Box<dyn Fn(&str) -> Vec<proc_macro2::TokenStream>> =
                                Box::new(|prefix| {
                                    named
                                        .named
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

                            match_innards.push(quote!(
                            #name::#vname { #(#idents: #self_vars),* } =>
                            #desc::#vname {
                                #(#idents: #self_vars.describe()),*
                            }));

                            delta_innards.push(quote!(
                            (#name::#vname { #(#idents: #self_vars),* },
                             #name::#vname { #(#idents: #other_vars),* }) =>
                                delta::Changed::Changed(
                                    delta::EnumChange::SameVariant(
                                        #change::#vname {
                                            #(#idents: #self_vars.delta(&#other_vars)),*
                                        }))));
                        }
                        Fields::Unnamed(unnamed) => {
                            let desc_decls: Vec<proc_macro2::TokenStream> = unnamed
                                .unnamed
                                .iter()
                                .map(|field| {
                                    let ty = &field.ty;
                                    quote!(<#ty as delta::Delta>::Desc)
                                })
                                .collect();
                            desc_innards.push(quote!(#vname(#(#desc_decls),*)));

                            let field_decls: Vec<proc_macro2::TokenStream> = unnamed
                                .unnamed
                                .iter()
                                .map(|field| {
                                    let ty = &field.ty;
                                    quote!(delta::Changed<<#ty as delta::Delta>::Change>)
                                })
                                .collect();
                            change_innards.push(quote!(#vname(#(#field_decls),*)));

                            let vars: Box<dyn Fn(&str) -> Vec<proc_macro2::TokenStream>> =
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

                            match_innards.push(quote!(
                                #name::#vname(#(#self_vars),*) =>
                                #desc::#vname(#(#self_vars.describe()),*)));

                            delta_innards.push(quote!(
                                (#name::#vname(#(#self_vars),*),
                                 #name::#vname(#(#other_vars),*)) =>
                                    delta::Changed::Changed(
                                        delta::EnumChange::SameVariant(
                                            #change::#vname(#(#self_vars.delta(&#other_vars)),*)))));
                        }
                        Fields::Unit => {
                            desc_innards.push(quote!(#vname));
                            change_innards.push(quote!(#vname));
                            match_innards.push(quote!(#name::#vname => #desc::#vname));
                            delta_innards.push(
                                quote!((#name::#vname, #name::#vname) => delta::Changed::Unchanged),
                            );
                        }
                    }
                }
            }
            delta_innards.push(quote!(
                (_, _) => delta::Changed::Changed(
                    delta::EnumChange::DiffVariant(
                        self.describe(), other.describe()))));
            let gen = quote! {
                // #[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
                #[derive(PartialEq, Debug)]
                #visibility enum #desc {
                    #(#desc_innards),*
                }

                // #[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
                #[derive(PartialEq, Debug)]
                #visibility enum #change {
                    #(#change_innards),*
                }

                impl delta::Delta for #name {
                    type Desc = #desc;

                    fn describe(&self) -> Self::Desc {
                        match self {
                            #(#match_innards),*
                        }
                    }

                    type Change = delta::EnumChange<Self::Desc, #change>;

                    fn delta(&self, other: &Self) -> delta::Changed<Self::Change> {
                        match (self, other) {
                            #(#delta_innards),*
                        }
                    }
                }
            };
            gen.into()
        }
        _ => {
            panic!("Delta derivation not yet implemented for unions");
        }
    }
}
