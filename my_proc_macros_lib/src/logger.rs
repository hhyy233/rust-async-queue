use quote::quote;
use syn::{parse_macro_input, ItemFn, Meta};

/// The args take a key value pair like `#[attrib_macro_logger(key = "value")]`.
pub fn attrib_proc_macro_impl_1(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = parse_macro_input!(args as syn::MetaNameValue);
    let item = parse_macro_input!(item as ItemFn);
    println!("meta => {:#?}", args);
    println!("item => {:#?}", item);
    quote! {}.into()
}

/// The args take a set of identifiers like `#[attrib_macro_logger(a, b, c)]`.
pub fn attrib_proc_macro_impl_2(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = parse_macro_input!(args as Meta);
    let item = parse_macro_input!(item as ItemFn);
    quote! {}.into()
}
