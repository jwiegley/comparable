use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, Fields, Ident};

#[proc_macro_derive(Delta)]
pub fn impl_delta(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = parse_macro_input!(input);
    let name = &ast.ident;
    let change = format_ident!("{}Change", name);
    if let Data::Struct(st) = &ast.data {
        match &st.fields {
            Fields::Named(named) => {
                let mut change_innards = quote!();
                for field in named.named.iter() {
                    let name: &syn::Ident = field.ident.as_ref().unwrap();
                    let ty = &field.ty;
                    let capitalized: &syn::Ident =
                        &Ident::new(&name.to_string().to_case(Case::Pascal), Span::call_site());
                    change_innards.extend(quote!(#capitalized(<#ty as Delta>::Change),));
                }

                let mut delta_innards = quote!();
                for field in named.named.iter() {
                    let name: &syn::Ident = field.ident.as_ref().unwrap();
                    let capitalized: &syn::Ident =
                        &Ident::new(&name.to_string().to_case(Case::Pascal), Span::call_site());
                    delta_innards
                        .extend(quote!(self.#name.delta(&other.#name).map(#change::#capitalized),));
                }

                let gen = quote! {
                    #[derive(PartialEq, Debug)]
                    enum #change {
                        #change_innards
                    }

                    impl Delta for #name {
                        type Desc = Vec<#change>;

                        fn describe(&self) -> Self::Desc {
                            #name::default().delta(self).unwrap_or_default()
                        }

                        type Change = Vec<#change>;

                        fn delta(&self, other: &Self) -> Option<Self::Change> {
                            let changes: Vec<#change> = vec![
                                #delta_innards
                            ]
                                .into_iter()
                                .flatten()
                                .collect();
                            if changes.is_empty() {
                                None
                            } else {
                                Some(changes)
                            }
                        }
                    }
                };
                gen.into()
            }
            Fields::Unnamed(_unnamed) => {
                panic!("Delta derivation not yet implemented for structs with unnamed fields")
            }
            Fields::Unit => panic!("Delta derivation not yet implemented for unit types"),
        }
    } else {
        panic!("Delta derivation not yet implemented for enums or unions");
    }
}
