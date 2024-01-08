extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

// using proc_macro_attribute to declare an attribute like procedural macro
#[proc_macro_attribute]
// _metadata is argument provided to macro call and _input is code to which attribute like macro attaches
pub fn my_custom_attribute(metadata: TokenStream, input: TokenStream) -> TokenStream {
    println!("{:#?}", metadata);
    println!();
    let input: ItemFn = parse_macro_input!(input as ItemFn);
    println!("{:#?}", input);
    // returing a simple TokenStream for Struct
    TokenStream::from(quote! {struct H{}})
}
