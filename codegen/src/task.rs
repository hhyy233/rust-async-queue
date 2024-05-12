use crate::{analyze::analyze, codegen::codegen, parse::parse};
use proc_macro2::TokenStream;

pub(crate) fn impl_macro(metadata: TokenStream, input: TokenStream) -> TokenStream {
    // println!("{:#?}", metadata);
    // println!();
    // let input: ItemFn = syn::parse2::<ItemFn>(input).unwrap();
    // println!("{:#?}", input);

    let ast = parse(metadata, input);
    let model = analyze(ast);
    codegen(model)
}

/*

ItemFn {
    attrs: [],
    vis: Inherited,
    sig: Signature {
        constness: None,
        asyncness: None,
        unsafety: None,
        abi: None,
        fn_token: Fn,
        ident: Ident {
            ident: "add",
            span: #0 bytes(85..88),
        },
        generics: Generics {
            lt_token: None,
            params: [],
            gt_token: None,
            where_clause: None,
        },
        paren_token: Paren,
        inputs: [
            Typed(
                PatType {
                    attrs: [],
                    pat: Ident(
                        PatIdent {
                            attrs: [],
                            by_ref: None,
                            mutability: None,
                            ident: Ident {
                                ident: "x",
                                span: #0 bytes(89..90),
                            },
                            subpat: None,
                        },
                    ),
                    colon_token: Colon,
                    ty: Path(
                        TypePath {
                            qself: None,
                            path: Path {
                                leading_colon: None,
                                segments: [
                                    PathSegment {
                                        ident: Ident {
                                            ident: "i32",
                                            span: #0 bytes(92..95),
                                        },
                                        arguments: None,
                                    },
                                ],
                            },
                        },
                    ),
                },
            ),
            Comma,
            Typed(
                PatType {
                    attrs: [],
                    pat: Ident(
                        PatIdent {
                            attrs: [],
                            by_ref: None,
                            mutability: None,
                            ident: Ident {
                                ident: "y",
                                span: #0 bytes(97..98),
                            },
                            subpat: None,
                        },
                    ),
                    colon_token: Colon,
                    ty: Path(
                        TypePath {
                            qself: None,
                            path: Path {
                                leading_colon: None,
                                segments: [
                                    PathSegment {
                                        ident: Ident {
                                            ident: "i32",
                                            span: #0 bytes(100..103),
                                        },
                                        arguments: None,
                                    },
                                ],
                            },
                        },
                    ),
                },
            ),
        ],
        variadic: None,
        output: Type(
            RArrow,
            Path(
                TypePath {
                    qself: None,
                    path: Path {
                        leading_colon: None,
                        segments: [
                            PathSegment {
                                ident: Ident {
                                    ident: "i32",
                                    span: #0 bytes(108..111),
                                },
                                arguments: None,
                            },
                        ],
                    },
                },
            ),
        ),
    },
    block: Block {
        brace_token: Brace,
        stmts: [
            Expr(
                Binary(
                    ExprBinary {
                        attrs: [],
                        left: Path(
                            ExprPath {
                                attrs: [],
                                qself: None,
                                path: Path {
                                    leading_colon: None,
                                    segments: [
                                        PathSegment {
                                            ident: Ident {
                                                ident: "x",
                                                span: #0 bytes(118..119),
                                            },
                                            arguments: None,
                                        },
                                    ],
                                },
                            },
                        ),
                        op: Add(
                            Add,
                        ),
                        right: Path(
                            ExprPath {
                                attrs: [],
                                qself: None,
                                path: Path {
                                    leading_colon: None,
                                    segments: [
                                        PathSegment {
                                            ident: Ident {
                                                ident: "y",
                                                span: #0 bytes(122..123),
                                            },
                                            arguments: None,
                                        },
                                    ],
                                },
                            },
                        ),
                    },
                ),
            ),
        ],
    },
}

*/
