mod attrs;
mod definition;
mod enums;
mod inputs;
mod outputs;
mod structs;
mod utils;

#[proc_macro_derive(
    Comparable,
    attributes(
        describe_type,
        describe_body,
        no_description,
        compare_default,
        comparable_public,
        comparable_private,
        comparable_ignore
    )
)]
pub fn comparable_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let inputs = crate::inputs::Inputs::from(&input);
    inputs.process_data().generate(&inputs).into()
}
