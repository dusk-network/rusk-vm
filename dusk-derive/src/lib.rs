extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;

struct Variable<T>(T);

pub fn variable(attr: TokenStream, item: TokenStream) -> TokenStream {}

#[proc_macro_attribute]
pub fn dusk_derive(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_string = attr.to_string();
    let result = quote! {
        pub struct ClientApi;

        impl ClientApi {
            pub fn debug() -> () {
                panic!("{:?}", #item_string);
            }
        }
    };
    result.into()
}
