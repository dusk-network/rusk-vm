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
    let q_impl = parse_macro_input!(input as syn::ItemImpl);
    let args = parse_macro_input!(attrs as Args);
    let q_fn_name = args.name;

    let q_impl_method = first_method_of_impl(q_impl.clone()).unwrap();
    let arg_types = non_self_argument_types(&q_impl_method.sig);

    let arg_t = arg_types.get(0).unwrap();
    let ret_t = return_type_of_sig(&q_impl_method.sig);
    let state_t = q_impl.self_ty.as_ref();

    let wrapper_fun_name = format_ident!("{}", q_fn_name);
    let scratch_name = format_ident!("scratch_{}", q_fn_name);
    let gen = quote! {

        impl Query for #arg_t {
            const NAME: &'static str = #q_fn_name;
            type Return = #ret_t;
        }

        #q_impl

        #[cfg(target_family = "wasm")]
        const _: () = {
            use rusk_uplink::{
                get_state_arg_store, q_return_store_ser,
                AbiStore, StoreContext
            };

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
pub fn transaction(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let t_impl = parse_macro_input!(input as syn::ItemImpl);
    let args = parse_macro_input!(attrs as Args);
    let t_fn_name = args.name;

    let t_impl_method = first_method_of_impl(t_impl.clone()).unwrap();
    let arg_types = non_self_argument_types(&t_impl_method.sig);

    let arg_t = arg_types.get(0).unwrap();
    let ret_t = return_type_of_sig(&t_impl_method.sig);
    let state_t = t_impl.self_ty.as_ref();

    let wrapper_fun_name = format_ident!("{}", t_fn_name);
    let scratch_name = format_ident!("scratch_{}", t_fn_name);
    let gen = quote! {

        impl Transaction for #arg_t {
            const NAME: &'static str = #t_fn_name;
            type Return = #ret_t;
        }

        #t_impl

        #[cfg(target_family = "wasm")]
        const _: () = {
            use rusk_uplink::{
                get_state_arg_store, t_return_store_ser,
                AbiStore, StoreContext
            };

            #[no_mangle]
            static mut #scratch_name: [u8; 512] = [0u8; 512];

            #[no_mangle]
            fn #wrapper_fun_name(written_state: u32, written_data: u32) -> [u32; 2] {
                let store =
                    StoreContext::new(AbiStore::new(unsafe { &mut #scratch_name }));
                let (mut state, arg): (#state_t, #arg_t) = unsafe {
                    get_state_arg_store(
                        written_state,
                        written_data,
                        &#scratch_name,
                        store.clone(),
                    )
                };

                let res: <#arg_t as Transaction>::Return =
                    state.apply(arg, store.clone());

                unsafe { t_return_store_ser(&state, &res, store) }
            }
        };
    };
    gen.into()
}

