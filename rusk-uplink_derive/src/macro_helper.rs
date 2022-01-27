use syn::{FnArg, Type};
use quote::{quote, ToTokens};

pub fn first_method_signature(
    an_impl: syn::ItemImpl,
) -> Option<syn::Signature> {
    for item in an_impl.items {
        if let syn::ImplItem::Method(method) = item
        {
            return Some(method.sig);
        }
    }
    None
}

pub fn non_self_argument_type(arg: &FnArg) -> Option<syn::Type> {
    let arg_ts_opt =  match arg {
        syn::FnArg::Receiver(r) => {
            let x = quote!(#r);
            println!("Receiver {}", x);
            return None
        },
        syn::FnArg::Typed(pt) => {
            let mut t = &pt.ty;
            t = match t.as_ref() {
                syn::Type::Reference(t) => &t.elem,
                _ => t,
            };
            let x = quote!(#t);
            println!("Typed {}", x);
            Some(x)
        },
    };
    arg_ts_opt.map(syn::Type::Verbatim)
}

pub fn arg_types(sig: &syn::Signature) -> Vec<Option<syn::Type>> {
    let mut v = Vec::new();
    for input in &sig.inputs {
        v.push(non_self_argument_type(&input))
    }
    v
}
