use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

mod ast_viz_debug;
mod logger;

#[proc_macro]
pub fn fn_macro_ast_viz_debug(input: TokenStream) -> TokenStream {
    ast_viz_debug::fn_proc_macro_impl(input)
}

#[proc_macro]
pub fn struct_macro_ast_viz_debug(input: TokenStream) -> TokenStream {
    ast_viz_debug::struct_proc_macro_impl(input)
}

#[proc_macro]
pub fn fn_proc_macro_impl(input: TokenStream) -> TokenStream {
    let _: ast_viz_debug::ManagerOfThingSyntaxInfo = parse_macro_input!(input);
    quote!(
        struct H {}
    )
    .into()
}

#[proc_macro_derive(Describe)]
pub fn derive_macro_describe(input: TokenStream) -> TokenStream {
    ast_viz_debug::derive_proc_macro_impl(input)
}

#[proc_macro_attribute]
pub fn attrib_macro_logger_1(args: TokenStream, item: TokenStream) -> TokenStream {
    logger::attrib_proc_macro_impl_1(args, item)
}

#[proc_macro_attribute]
pub fn attrib_macro_logger_2(args: TokenStream, item: TokenStream) -> TokenStream {
    logger::attrib_proc_macro_impl_2(args, item)
}
