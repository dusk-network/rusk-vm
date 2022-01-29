extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;


mod macro_helper;
use macro_helper::*;

mod args;
use args::*;

#[proc_macro_attribute]
pub fn query(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let my_impl = parse_macro_input!(input as syn::ItemImpl);
    let args = parse_macro_input!(attrs as Args);
    let fn_name = args.name;

    let my_method = first_method_of_impl(my_impl.clone()).unwrap();
    let arg_types = non_self_argument_types(&my_method.sig);

    let arg_t = arg_types.get(0).unwrap();
    let ret_t = return_type_of_sig(&my_method.sig);
    let state_t = my_impl.self_ty.as_ref();

    let wrapper_fun_name = format_ident!("{}", fn_name);
    let scratch_name = format_ident!("scratch_{}", fn_name);
    let gen = quote! {

        impl Query for #arg_t {
            const NAME: &'static str = #fn_name;
            type Return = #ret_t;
        }

        #my_impl

        #[cfg(target_family = "wasm")]
        const _: () = {
            use rusk_uplink::framing_imports;
            use rusk_uplink::{
                get_state, get_state_arg, get_state_arg_store, q_return,
                q_return_store_ser, q_handler,
                q_handler_store_ser,
                AbiStore, StoreContext
            };
            use microkelvin::{OffsetLen, StoreRef};

            #[no_mangle]
            static mut #scratch_name: [u8; 512] = [0u8; 512];

            #[no_mangle]
            fn #wrapper_fun_name(written_state: u32, written_data: u32) -> u32 {
                let store =
                    StoreContext::new(AbiStore::new(unsafe { &mut #scratch_name }));
                let (state, arg): (#state_t, #arg_t) = unsafe {
                    get_state_arg_store(
                        written_state,
                        written_data,
                        &#scratch_name,
                        store.clone(),
                    )
                };

                let res: <#arg_t as Query>::Return =
                    state.execute(arg, store.clone());

                unsafe { q_return_store_ser(&res, store) }
            }
        };
    };
    gen.into()
}

#[proc_macro_attribute]
pub fn transaction(_attrs: TokenStream, input: TokenStream) -> TokenStream {
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
            let mut t = &pt.ty;
            t = match t.as_ref() {
                syn::Type::Reference(t) => &t.elem,
                _ => t,
            };
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
        fn #wrapper_fun_name(written_state: u32, written_data: u32) -> [u32; 2] {
            let store =
                StoreContext::new(AbiStore::new(unsafe { &mut SCRATCH }));
            let (mut state, arg): (#state_t, #arg_t) = unsafe {
                get_state_arg_store(written_state, written_data, &SCRATCH, store.clone())
            };

            let res: #ret_t = #fn_name(&mut state, arg, store.clone());

            unsafe { t_return_store_ser(&state, &res, store) }
        }

    };
    gen.into()
}

