use proc_macro2::TokenStream;
use proc_macro_error::{abort, abort_call_site};
use syn::{Expr, Item, ItemFn};

pub type Ast = ItemFn;

pub(crate) fn parse(args: TokenStream, item: TokenStream) -> Ast {
    const ERROR: &str = "this attribute takes no arguments";
    const HELP: &str = "use `#[contracts]`";

    if !args.is_empty() {
        if let Ok(expr) = syn::parse2::<Expr>(args) {
            // ../tests/ui/has-expr-argument.rs
            abort!(expr, ERROR; help = HELP)
        } else {
            // ../tests/ui/has-arguments.rs
            abort_call_site!(ERROR; help = HELP)
        }
    }

    match syn::parse2::<Item>(item) {
        Ok(Item::Fn(item)) => item,
        Ok(item) => {
            // ../tests/ui/item-is-not-a-function.rs
            abort!(
                item,
                "item is not a function";
                help = "`#[contracts]` can only be used on functions"
            )
        }
        Err(_) => unreachable!(), // ?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn get_item() {
        let ast = parse(
            quote!(),
            quote!(
                fn add(x: i32, y: i32) -> i32 {
                    x + y
                }
            ),
        );
        assert_eq!("add", ast.sig.ident.to_string())
    }
}
