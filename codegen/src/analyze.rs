use proc_macro2::{Span, TokenStream};
use proc_macro_error::{abort, abort_call_site};
use quote::quote;
use syn::{parse_quote, Block, Expr, FnArg, Ident, ReturnType, Type};

use crate::Ast;

pub(crate) struct Model {
    name: String,
    ident: Ident,
    param_ident: Ident,
    input_args: Vec<FnArg>,
    return_type: Option<Type>,
    block: Block,
    krate: TokenStream,
}

pub(crate) fn analyze(ast: Ast) -> Model {
    let ident = ast.sig.ident.clone();
    let name = ident.to_string();
    let param_ident = get_param_ident(&name);

    let input_args: Vec<FnArg> = ast.sig.inputs.clone().into_iter().collect();
    let mut return_type = None;
    if let ReturnType::Type(_, ref ty) = ast.sig.output {
        return_type = Some((**ty).clone());
    };
    let block = (*ast.block).clone();

    Model {
        name,
        ident,
        param_ident,
        input_args,
        return_type,
        block,
        krate: quote!(::rust_async_queue),
    }
}

fn get_param_name(name: &String) -> String {
    format!("{name}Params")
}

fn get_param_ident(name: &String) -> Ident {
    let param_name = get_param_name(name);
    syn::Ident::new(&param_name[..], Span::call_site())
}

fn extract_arg_ident(args: &Vec<FnArg>) -> Vec<Ident> {
    args.into_iter()
        .map(|fa| match fa {
            FnArg::Typed(pt) => match *pt.pat {
                syn::Pat::Ident(ref pat) => pat.ident.clone(),
                _ => abort_call_site!("not an ident"),
            },
            _ => abort_call_site!("not a type argument"),
        })
        .collect()
}

fn construct_assignments(args: &Vec<FnArg>) -> Vec<Expr> {
    args.into_iter()
        .map(|fa| match fa {
            FnArg::Typed(pt) => match *pt.pat {
                syn::Pat::Ident(ref pat) => {
                    let id = &pat.ident;
                    parse_quote! {
                        let #id = params.#id
                    }
                }
                ref default => abort!(default, "not an ident"),
            },
            FnArg::Receiver(rc) => abort!(rc, "not a type argument"),
        })
        .collect()
}

impl Model {
    pub fn build_param_struct(&self) -> TokenStream {
        let krate = &self.krate;
        let param_ident = &self.param_ident;
        let input_args = &self.input_args;
        quote! {
            #[allow(non_camel_case_types)]
            #[derive(
                Clone,
                #krate::export::Serialize, #krate::export::Deserialize,
            )]
            struct #param_ident {
                #(#input_args,)*
            }
        }
    }

    pub fn build_struct(&self) -> TokenStream {
        let ident = &self.ident;
        let param_ident = &self.param_ident;
        quote! {
            struct #ident {
                params: #param_ident,
            }
        }
    }

    pub fn build_struct_impl(&self) -> TokenStream {
        let krate = &self.krate;

        let ident = &self.ident;
        let param_ident = &self.param_ident;

        let input_args = &self.input_args;
        let input_idents = extract_arg_ident(input_args);

        let return_type = &self.return_type;
        let block = &self.block;
        let assignment = construct_assignments(&self.input_args);

        quote! {
            impl #ident {
                fn new( #(#input_args),* ) -> #krate::app::signature::Signature<Self> {
                    #krate::app::signature::Signature::<Self>::new(
                        #param_ident {
                            #(#input_idents),*
                        }
                    )
                }
                fn _run(params: #param_ident) -> #return_type {
                    #(#assignment;)*
                    #block
                }
            }
        }
    }

    pub fn build_struct_impl_for_task(&self) -> TokenStream {
        let krate = &self.krate;

        let name = &self.name;
        let ident = &self.ident;
        let param_ident = &self.param_ident;
        let return_type = &self.return_type;

        quote! {
            impl #krate::app::task::AQTask for #ident {
                const NAME: &'static str = #name;
                type Params = #param_ident;
                type Returns = #return_type;

                fn run(&self) -> Self::Returns {
                    #ident::_run(self.params.clone())
                }
                fn from_params(params: Self::Params) -> Self {
                    Self { params }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::{parse2, parse_quote, ItemImpl, ItemStruct};

    use super::*;

    #[test]
    fn test_model() {
        let ast: Ast = parse_quote!(
            fn f(x: bool) -> bool {}
        );
        let model = analyze(ast);
        assert_eq!("f".to_string(), model.name);
        assert_eq!("f".to_string(), model.ident.to_string());
        let input_args = model.input_args.clone();
        assert_eq!(1, input_args.len());

        let input = match input_args[0].clone() {
            FnArg::Typed(x) => x,
            default => panic!("expect fn arg, but got {:?}", default),
        };
        let expected: syn::Pat = parse_quote!(x);
        assert_eq!(expected, *input.pat);

        let expected: syn::Type = parse_quote!(bool);
        assert_eq!(expected, *input.ty);

        assert_eq!(Some(expected), model.return_type);

        let expected: syn::Block = parse_quote!({});
        assert_eq!(expected, model.block);
    }

    #[test]
    fn test_construct_param_struct() {
        let ast = parse_quote!(
            fn add(x: i32, y: i32) -> i32 {
                x + y
            }
        );
        let model = analyze(ast);
        let output = model.build_param_struct();

        let expected: ItemStruct = parse_quote!(
            #[allow(non_camel_case_types)]
            #[derive(
                Clone,
                ::rust_async_queue::export::Serialize,
                ::rust_async_queue::export::Deserialize,
            )]
            struct addParams {
                x: i32,
                y: i32,
            }
        );
        let expected_stream = quote! {#expected};

        let actual = parse2::<ItemStruct>(output.clone()).unwrap();
        assert_eq!(
            expected, actual,
            "want {}\n got {}\n",
            expected_stream, output
        )
    }

    #[test]
    fn test_construct_struct() {
        let ast = parse_quote!(
            fn add(x: i32, y: i32) -> i32 {
                x + y
            }
        );
        let model = analyze(ast);
        let output = model.build_struct();

        let expected: ItemStruct = parse_quote!(
            struct add {
                params: addParams,
            }
        );
        let expected_stream = quote! {#expected};

        let actual = parse2::<ItemStruct>(output.clone()).unwrap();
        assert_eq!(
            expected, actual,
            "want {}\n got {}\n",
            expected_stream, output
        )
    }

    #[test]
    fn test_construct_assignment() {
        let ast = parse_quote!(
            fn add(x: i32, y: i32) -> i32 {
                x + y
            }
        );
        let model = analyze(ast);
        let output = construct_assignments(&model.input_args);

        let expected: Vec<Expr> = vec![
            parse_quote!(let x = params.x),
            parse_quote!(let y = params.y),
        ];
        assert_eq!(
            expected, output,
            "want {:#?}\n got {:#?}\n",
            expected, output
        )
    }

    #[test]
    fn test_build_struct_impl() {
        let ast = parse_quote!(
            fn add(x: i32, y: i32) -> i32 {
                x + y
            }
        );
        let model = analyze(ast);
        let output = model.build_struct_impl();

        let expected: ItemImpl = parse_quote! {
            impl add {
                fn new(x: i32, y: i32) -> ::rust_async_queue::app::signature::Signature<Self> {
                    ::rust_async_queue::app::signature::Signature::<Self>::new(addParams { x, y })
                }
                fn _run(params: addParams) -> i32 {
                    let x = params.x;
                    let y = params.y;
                    {x + y}
                }
            }
        };
        let expected_stream = quote! {#expected};

        let actual = parse2::<ItemImpl>(output.clone()).unwrap();
        assert_eq!(
            expected, actual,
            "want {}\n got {}\n",
            expected_stream, output
        )
    }

    #[test]
    fn test_build_struct_impl_for_task() {
        let ast = parse_quote!(
            fn add(x: i32, y: i32) -> i32 {
                x + y
            }
        );
        let model = analyze(ast);
        let output = model.build_struct_impl_for_task();

        let expected: ItemImpl = parse_quote! {
            impl ::rust_async_queue::app::task::AQTask for add {
                const NAME: &'static str = "add";
                type Params = addParams;
                type Returns = i32;

                fn run(&self) -> Self::Returns {
                    add::_run(self.params.clone())
                }
                fn from_params(params: Self::Params) -> Self {
                    Self { params }
                }
            }
        };
        let expected_stream = quote! {#expected};

        let actual = parse2::<ItemImpl>(output.clone()).unwrap();
        assert_eq!(
            expected, actual,
            "want {}\n got {}\n",
            expected_stream, output
        )
    }
}
