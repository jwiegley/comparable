use quote::quote;
use syn::spanned::Spanned;

pub fn impl_for_tuple(tup: syn::TypeTuple) -> proc_macro2::TokenStream {
    let field_types = tup.elems.iter().map(|e| quote!(#e)).collect::<Vec<_>>();

    let enumerated_elems: Vec<_> = tup.elems.iter().enumerate().collect();
    let results = enumerated_elems
        .iter()
        .map(|(i, t)| syn::Ident::new(&format!("res{}", i,)[..], t.span()))
        .collect::<Vec<_>>();
    let indexes = enumerated_elems
        .iter()
        .map(|(i, _)| {
            let i = syn::Index::from(*i);
            quote!(#i)
        })
        .collect::<Vec<_>>();
    quote! {
        #[automatically_derived]
        impl <#(#field_types: Comparable,)*> Comparable for (#(#field_types,)*) {
            type Desc = (#(#field_types::Desc),*);

            fn describe(&self) -> Self::Desc {
                (#(self.#indexes.describe()),*)
            }

            type Change = (#(Changed<#field_types::Change>),*);

            fn comparison(&self, other: &Self) -> Changed<Self::Change> {
                let mut has_change = false;
                #(
                    let #results = self.#indexes.comparison(&other.#indexes);
                    has_change = has_change || !#results.is_unchanged();
                )*
                if has_change {
                    Changed::Changed((#(#results),*))
                } else {
                    Changed::Unchanged
                }
            }
        }
    }
}
