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
pub fn execute(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let q_impl = parse_macro_input!(input as syn::ItemImpl);
    let args = parse_macro_input!(attrs as Args);
    let q_fn_name = args.name;

    let q_impl_method = first_method_of_impl(q_impl.clone()).unwrap();
    let arg_types = non_self_argument_types(&q_impl_method.sig);

    let arg_t = arg_types.get(0).unwrap();
    let _ret_t = return_type_of_sig(&q_impl_method.sig);
    let state_t = q_impl.self_ty.as_ref();

    let wrapper_fun_name = format_ident!("{}", q_fn_name);
    let scratch_name = format_ident!("scratch");
    let gen = quote! {

        #q_impl

        #[cfg(target_family = "wasm")]
        const _: () = {
            use rusk_uplink::{
                get_state_arg, q_return, AbiStore, StoreContext
            };

            #[no_mangle]
            fn #wrapper_fun_name(written_state: u32, written_data: u32) -> u32 {
                let (state_arg, mut rest) = unsafe { crate::scratch_mod::#scratch_name.split_at_mut(written_data as usize) };
                let store =
                    StoreContext::new(AbiStore::new(unsafe { &mut rest }, unsafe { &mut crate::scratch_mod::#scratch_name }, written_data as usize));
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

                let scratch_mem = unsafe { crate::scratch_mod::#scratch_name.as_mut_slice() };
                let store = StoreContext::new(AbiStore::new(scratch_mem, unsafe { &mut crate::scratch_mod::#scratch_name }, 0));
                unsafe { q_return(&res, store) }
            }
        };
    };
    gen.into()
}

#[proc_macro_attribute]
pub fn apply(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let t_impl = parse_macro_input!(input as syn::ItemImpl);
    let args = parse_macro_input!(attrs as Args);
    let t_fn_name = args.name;

    let t_impl_method = first_method_of_impl(t_impl.clone()).unwrap();
    let arg_types = non_self_argument_types(&t_impl_method.sig);

    let arg_t = arg_types.get(0).unwrap();
    let _ret_t = return_type_of_sig(&t_impl_method.sig);
    let state_t = t_impl.self_ty.as_ref();

    let wrapper_fun_name = format_ident!("{}", t_fn_name);
    let scratch_name = format_ident!("scratch");
    let gen = quote! {

        #t_impl

        #[cfg(target_family = "wasm")]
        const _: () = {
            use rusk_uplink::{
                get_state_arg, t_return, AbiStore, StoreContext
            };

            #[no_mangle]
            fn #wrapper_fun_name(written_state: u32, written_data: u32) -> [u32; 2] {
                let (state_arg, mut rest) = unsafe { crate::scratch_mod::#scratch_name.split_at_mut(written_data as usize) };
                let store =
                    StoreContext::new(AbiStore::new(unsafe { &mut rest }, unsafe { &mut crate::scratch_mod::#scratch_name }, written_data as usize));
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

                let scratch_mem = unsafe { crate::scratch_mod::#scratch_name.as_mut_slice() };
                let store = StoreContext::new(AbiStore::new(scratch_mem, unsafe { &mut crate::scratch_mod::#scratch_name }, 0));
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
pub fn query(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let arg_struct = parse_macro_input!(input as syn::ItemStruct);
    let args = parse_macro_input!(attrs as DeriveArgs);
    generate_struct_derivations(arg_struct, args.derive_new)
}

#[proc_macro_attribute]
pub fn transaction(attrs: TokenStream, input: TokenStream) -> TokenStream {
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

#[proc_macro_attribute]
pub fn init(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    let init_impl = parse_macro_input!(input as syn::ItemFn);

    let gen = quote! {
        #[cfg(target_family = "wasm")]
        mod scratch_mod {
            extern crate alloc;
            use alloc::vec::Vec;
            #[no_mangle]
            pub static mut scratch: Vec<u8> = Vec::<u8>::new();

            #[no_mangle]
            pub fn grow_scratch(sz: u32) -> u32 {
                const MIN_GROW_BY: usize = 128;
                unsafe {
                    if (sz as usize > scratch.len()) || (scratch.len() == 0) {
                        let len = core::cmp::max(sz as usize, scratch.len()) + MIN_GROW_BY;
                        scratch.resize(len, 0u8);
                    }
                    scratch.as_mut_ptr() as *mut _ as u32
                }
            }

            #[no_mangle]
            #init_impl
        }
    };
    gen.into()
}
