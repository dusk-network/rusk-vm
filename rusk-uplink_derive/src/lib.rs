extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
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
pub fn query_gen(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let my_fn: syn::ItemFn = parse_macro_input!(input as syn::ItemFn);
    let fn_name = my_fn.sig.ident.clone();
    // let ret_type = my_fn.sig.output.clone();
    let gen = quote! {
        impl Query for XiongMao4 {
            const NAME: &'static str = stringify!(#fn_name);
            type Return = ();
        }
        #my_fn
    };
    gen.into()
}