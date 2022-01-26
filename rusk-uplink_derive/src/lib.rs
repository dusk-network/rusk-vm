extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
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
pub fn query(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    let my_fn: syn::ItemFn = parse_macro_input!(input as syn::ItemFn);
    let fn_name = my_fn.sig.ident.clone();
    let ret_obj = my_fn.sig.output.clone();
    let ret = match ret_obj {
        syn::ReturnType::Default => quote::quote_spanned!(my_fn.sig.paren_token.span=> ()),
        syn::ReturnType::Type(_, r) => quote!(#r),
    };
    let ret_t = syn::Type::Verbatim(ret);

    let mut arg_iter = my_fn.sig.inputs.iter();
    let state_obj = arg_iter.next().unwrap();
    let state =  match state_obj {
        syn::FnArg::Receiver(_) => quote::quote_spanned!(my_fn.sig.paren_token.span=> ()),
        syn::FnArg::Typed(pt) => {
            let t = &pt.ty;
            quote!(#t)
        },
    };
    let state_t = syn::Type::Verbatim(state);

    let arg_obj = arg_iter.next().unwrap();
    let arg =  match arg_obj {
        syn::FnArg::Receiver(_) => quote::quote_spanned!(my_fn.sig.paren_token.span=> ()),
        syn::FnArg::Typed(pt) => {
            let t = &pt.ty;
            quote!(#t)
        },
    };
    let arg_t = syn::Type::Verbatim(arg);

    let wrapper_fun_name = format_ident!("_{}", fn_name);

    let gen = quote! {
        // impl Query for #arg_t {
        //     const NAME: &'static str = stringify!(#fn_name);
        //     type Return = #ret_t;
        // }

        #my_fn

        #[no_mangle]
        fn #wrapper_fun_name(written_state: u32, written_data: u32) -> u32 {
            let (state, arg): (#state_t, #arg_t) = unsafe {
                get_state_arg(written_state, written_data, &SCRATCH)
            };

            let store =
                StoreContext::new(AbiStore::new(unsafe { &mut SCRATCH }));
            let res: #ret_t = #fn_name(state, arg, store);

            unsafe { q_return(&res, &mut SCRATCH) }
        }

    };
    gen.into()
}