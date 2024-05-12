mod analyze;
mod codegen;
mod parse;
mod task;

use analyze::Model;
use parse::Ast;
use proc_macro::TokenStream;

// using proc_macro_attribute to declare an attribute like procedural macro
// _metadata is argument provided to macro call and _input is code to which attribute like macro attaches
#[proc_macro_attribute]
pub fn task(metadata: TokenStream, input: TokenStream) -> TokenStream {
    task::impl_macro(metadata.into(), input.into()).into()
}
