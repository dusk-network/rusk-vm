extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;

mod macro_helper;
use macro_helper::*;

mod args;
use args::*;

mod derive_args;
use derive_args::*;

#[proc_macro_attribute]
pub fn query(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let q_impl = parse_macro_input!(input as syn::ItemImpl);
    let args = parse_macro_input!(attrs as Args);
    let q_fn_name = args.name;
    let buf_size = args.buf;

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
                get_state_arg, q_return, AbiStore, StoreContext
            };

            #[no_mangle]
            static mut #scratch_name: [u8; #buf_size] = [0u8; #buf_size];

            #[no_mangle]
            fn #wrapper_fun_name(written_state: u32, written_data: u32) -> u32 {
                let (state_arg, mut rest) = unsafe { #scratch_name.split_at_mut(written_data as usize) };
                let store =
                    StoreContext::new(AbiStore::new(unsafe { &mut rest }));
                let (state, arg): (#state_t, #arg_t) = unsafe {
                    get_state_arg(
                        written_state,
                        written_data,
                        &state_arg,
                        store.clone(),
                    )
                };

                let res: <#arg_t as Query>::Return =
                    state.execute(arg, store.clone());

                let scratch = unsafe { &mut #scratch_name[..] };
                let store = StoreContext::new(AbiStore::new(scratch));
                unsafe { q_return(&res, store) }
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
    let buf_size = args.buf;

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
                get_state_arg, t_return, AbiStore, StoreContext
            };

            #[no_mangle]
            static mut #scratch_name: [u8; #buf_size] = [0u8; #buf_size];

            #[no_mangle]
            fn #wrapper_fun_name(written_state: u32, written_data: u32) -> [u32; 2] {
                let (state_arg, mut rest) = unsafe { #scratch_name.split_at_mut(written_data as usize) };
                let store =
                    StoreContext::new(AbiStore::new(unsafe { &mut rest }));
                let (mut state, arg): (#state_t, #arg_t) = unsafe {
                    get_state_arg(
                        written_state,
                        written_data,
                        &state_arg,
                        store.clone(),
                    )
                };

                let res: <#arg_t as Transaction>::Return =
                    state.apply(arg, store.clone());

                let scratch = unsafe { &mut #scratch_name[..] };
                let store = StoreContext::new(AbiStore::new(scratch));
                unsafe { t_return(&state, &res, store) }
            }
        };
    };
    gen.into()
}

fn generate_struct_derivations(
    arg_struct: syn::ItemStruct,
    derive_new: bool,
) -> TokenStream {
    let gen = if derive_new {
        quote! {
            #[derive(derive_new::new, Clone, Debug, Default, Archive, Serialize, Deserialize)]
            #arg_struct
        }
    } else {
        quote! {
            #[derive(Clone, Default, Archive, Serialize, Deserialize)]
            #arg_struct
        }
    };
    gen.into()
}

#[proc_macro_attribute]
pub fn argument(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let arg_struct = parse_macro_input!(input as syn::ItemStruct);
    let args = parse_macro_input!(attrs as DeriveArgs);
    generate_struct_derivations(arg_struct, args.derive_new)
}

#[proc_macro_attribute]
pub fn state(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let arg_struct = parse_macro_input!(input as syn::ItemStruct);
    let args = parse_macro_input!(attrs as DeriveArgs);
    generate_struct_derivations(arg_struct, args.derive_new)
}
