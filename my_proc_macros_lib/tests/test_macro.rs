use async_trait::async_trait;
use my_proc_macros_lib::{
    attrib_macro_logger_1, fn_macro_ast_viz_debug, fn_proc_macro_impl, struct_macro_ast_viz_debug,
    Describe,
};
use r3bl_rs_utils_macro::make_struct_safe_to_share_and_mutate;

// #[test]
// fn test_proc_macro() {
//     // fn_macro_ast_viz_debug!();
//     // assert_eq!(foo(), 42);
//     // struct_macro_ast_viz_debug!()
//     fn_proc_macro_impl! {
//         named MyMapManager<K, V>
//         where K: Default + Send + Sync + 'static, V: Default + Send + Sync + 'static
//         containing my_map
//         of_type std::collections::HashMap<K, V>
//     }
// }

/*
Debug::ast => ItemFn { attrs: [], vis: Visibility::Inherited, sig: Signature { constness: None, asyncness: None, unsafety: None, abi: None, fn_token: Fn, ident: Ident { ident: "foo", span: #6 bytes(84..109) }, generics: Generics { lt_token: None, params: [], gt_token: None, where_clause: None }, paren_token: Paren, inputs: [], variadic: None, output: ReturnType::Type(RArrow, Type::Path { qself: None, path: Path { leading_colon: None, segments: [PathSegment { ident: Ident { ident: "u32", span: #6 bytes(84..109) }, arguments: PathArguments::None }] } }) }, block: Block { brace_token: Brace, stmts: [Stmt::Expr(Expr::Lit { attrs: [], lit: Lit::Int { token: 42 } }, None)] } }
=> ast_item_fn < attrs.len:0, vis:inherited, sig:'foo' stmt: '42' >
*/

// #[test]
// fn test_manager() {
//     make_struct_safe_to_share_and_mutate! {
//       named MyMapManager<K, V>
//       where K: Default + Send + Sync + 'static, V: Default + Send + Sync + 'static
//       containing my_map
//       of_type std::collections::HashMap<K, V>
//     }
// }

// #[test]
// fn test_proc_macro() {
//     #[derive(Describe)]
//     struct MyStruct<T> {
//         my_string: String,
//         my_enum: MyEnum,
//         my_number: T,
//     }

//     #[derive(Describe)]
//     enum MyEnum {
//         MyVariant1,
//     }

//     let foo = MyStruct {
//         my_string: "Hello".to_string(),
//         my_enum: MyEnum::MyVariant1,
//         my_number: 42,
//     };
//     let foo = foo.describe();
//     assert_eq!(
//         foo,
//         "MyStruct is a struct with these named fields: my_string, my_enum, my_number"
//     );
// }

// #[attrib_macro_logger_1(key = "value")]
// pub fn some_annotated_function() {
//     /* ... */
// }
