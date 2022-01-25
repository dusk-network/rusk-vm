extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{AttributeArgs, DeriveInput};
use syn::parse_macro_input;


#[proc_macro_derive(HelloMacro, attributes(name))]
pub fn hello_macro_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_hello_macro(&ast)
}

fn impl_hello_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl HelloMacro for #name {
            const NAME: &'static str = "abc";
            fn hello_macro() {
                println!("Hello, Macro! My MMX name is {}!", stringify!(#name));
            }
        }
    };
    gen.into()
}


#[proc_macro_attribute]
pub fn query_gen(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    let my_fn: syn::ItemFn = parse_macro_input!(input as syn::ItemFn);
    let fn_name = my_fn.sig.ident.clone();
    let ret_type = my_fn.sig.output.clone();
    let ret = match ret_type {
        syn::ReturnType::Default => quote::quote_spanned!(my_fn.sig.paren_token.span=> ()),
        syn::ReturnType::Type(_, r) => quote!(#r),
    };
    let ret_t = syn::Type::Verbatim(ret);
    // let arg = my_fn.sig.inputs.first().unwrap().to_token_stream();
    // let arg_t = syn::Type::Verbatim(arg);
    let gen = quote! {
        impl Query for XiongMao4 {
            const NAME: &'static str = stringify!(#fn_name);
            type Return = #ret_t;
        }
        #my_fn
    };
    gen.into()
}