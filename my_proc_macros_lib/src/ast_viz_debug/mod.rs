use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use r3bl_rs_utils_core::{style_primary, style_prompt};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_str,
    Data::{Enum, Struct, Union},
    DataEnum, DataStruct, DataUnion, DeriveInput,
    Fields::{Named, Unit, Unnamed},
    FieldsNamed, FieldsUnnamed, Generics, Ident, ItemFn, ItemStruct, Result, Type, WhereClause,
};

/// https://docs.rs/syn/latest/syn/macro.parse_macro_input.html
pub fn fn_proc_macro_impl(_input: TokenStream) -> TokenStream {
    // let output_token_stream_str = "struct M<K, V> {}";
    let output_token_stream_str = "fn foo() -> u32 { 42 }";
    let output = output_token_stream_str.parse().unwrap();

    let ast_item_fn: ItemFn = parse_str::<ItemFn>(output_token_stream_str).unwrap();
    viz_ast(ast_item_fn);

    output
}

fn viz_ast(ast: ItemFn) {
    // Simply dump the AST to the console.
    let ast_clone = ast.clone();
    eprintln!("{} => {:?}", style_primary("Debug::ast"), ast_clone);

    // Parse AST to dump some items to the console.
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = ast;

    eprintln!(
        "{} ast_item_fn < attrs.len:{}, vis:{}, sig:'{}' stmt: '{}' >",
        style_primary("=>"),
        style_prompt(&attrs.len().to_string()),
        style_prompt(match vis {
            syn::Visibility::Public(_) => "public",
            // syn::Visibility::Crate(_) => "crate",
            syn::Visibility::Restricted(_) => "restricted",
            syn::Visibility::Inherited => "inherited",
        }),
        style_prompt(&sig.ident.to_string()),
        style_prompt(&match block.stmts.first() {
            Some(stmt) => {
                let expr_str = stmt.to_token_stream().to_string().clone();
                expr_str
            }
            None => "empty".to_string(),
        }),
    );
}

pub fn struct_proc_macro_impl(_input: TokenStream) -> TokenStream {
    let output_token_stream_str = "struct Foo<A> { x: A }";
    let output = output_token_stream_str.parse().unwrap();

    let ast_item: ItemStruct = parse_str::<ItemStruct>(output_token_stream_str).unwrap();
    viz_ast_struct(ast_item);

    output
}

fn viz_ast_struct(ast: ItemStruct) {
    // Simply dump the AST to the console.
    let ast_clone = ast.clone();
    eprintln!("{} => {:?}", style_primary("Debug::ast"), ast_clone);
}

pub struct ManagerOfThingSyntaxInfo {}

mod kw {
    syn::custom_keyword!(named);
    syn::custom_keyword!(containing);
    syn::custom_keyword!(of_type);
}

/// [Parse docs](https://docs.rs/syn/latest/syn/parse/index.html)
impl Parse for ManagerOfThingSyntaxInfo {
    fn parse(input: ParseStream) -> Result<Self> {
        // println!("{:#?}", input);
        input.parse::<kw::named>()?;

        let manager_type: Type = input.parse()?;
        println!("manager_type => {:#?}", manager_type);

        let where_clause = input.parse::<WhereClause>()?;
        println!("where => {:#?}", where_clause);

        input.parse::<kw::containing>()?;
        let property_name_ident: Ident = input.parse()?;
        println!("name => {:#?}", property_name_ident);

        input.parse::<kw::of_type>()?;
        let thing_type: Type = input.parse()?;
        println!("type => {:#?}", thing_type);

        Ok(ManagerOfThingSyntaxInfo {})
    }
}

pub fn derive_proc_macro_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let di: DeriveInput = parse_macro_input!(input);
    println!("{:#?}", di);
    let DeriveInput {
        ident,
        data,
        generics,
        ..
    } = di;
    let description = match data {
        Struct(my_struct) => gen_description_str_for_struct(my_struct),
        Enum(my_enum) => gen_description_str_for_enum(my_enum),
        Union(my_union) => gen_description_str_for_union(my_union),
    };

    let parsed_generics = parse_generics(&generics);
    match parsed_generics {
        Some(ref _generic_ident) => {
            quote! {
              impl <#parsed_generics> #ident <#parsed_generics> {
                fn describe(&self) -> String {
                  let mut string = String::from(stringify!(#ident));
                  string.push_str(" is ");
                  string.push_str(#description);
                  string
                }
              }
            }
            .into() // Convert from proc_macro2::TokenStream to TokenStream.
        }
        None => {
            quote! {
              impl #ident  {
                fn describe(&self) -> String {
                  let mut string = String::from(stringify!(#ident));
                  string.push_str(" is ");
                  string.push_str(#description);
                  string
                }
              }
            }
            .into() // Convert from proc_macro2::TokenStream to TokenStream.
        }
    }
}

fn gen_description_str_for_struct(my_struct: DataStruct) -> String {
    match my_struct.fields {
        Named(fields) => handle_named_fields(fields),
        Unnamed(fields) => handle_unnamed_fields(fields),
        Unit => handle_unit(),
    }
}

fn handle_named_fields(fields: FieldsNamed) -> String {
    let my_named_field_idents = fields.named.iter().map(|it| &it.ident);
    format!(
        "a struct with these named fields: {}",
        quote! {#(#my_named_field_idents), *}
    )
}

fn handle_unnamed_fields(fields: FieldsUnnamed) -> String {
    let my_unnamed_fields_count = fields.unnamed.iter().count();
    format!("a struct with {} unnamed fields", my_unnamed_fields_count)
}

fn handle_unit() -> String {
    format!("a unit struct")
}

fn gen_description_str_for_enum(my_enum: DataEnum) -> String {
    let my_variant_idents = my_enum.variants.iter().map(|it| &it.ident);
    format!(
        "an enum with these variants: {}",
        quote! {#(#my_variant_idents),*}
    )
}

fn gen_description_str_for_union(my_union: DataUnion) -> String {
    handle_named_fields(my_union.fields)
}

fn parse_generics(generics: &Generics) -> Option<Ident> {
    if let Some(generic_param) = generics.params.first() {
        // https://docs.rs/syn/latest/syn/enum.GenericParam.html
        match generic_param {
            syn::GenericParam::Type(ref param) => Some(param.ident.clone()),
            syn::GenericParam::Lifetime(_) => unimplemented!(),
            syn::GenericParam::Const(_) => unimplemented!(),
        }
    } else {
        None
    }
}
