mod tuple;

use quote::quote;

#[proc_macro]
pub fn impl_comparable_for_tuple(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ty: syn::Type =
        syn::parse(input).expect("Please pass a tuple type to impl_comparable_for_tuple");
    match ty {
        syn::Type::Tuple(t) => {
            let res = tuple::impl_for_tuple(t);
            quote!(#res).into()
        }
        _ => panic!("error in impl_comparable_for_tuple: only tuples are supported"),
    }
}
