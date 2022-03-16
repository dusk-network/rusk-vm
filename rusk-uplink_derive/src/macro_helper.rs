// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use quote::quote;
use syn::FnArg;

pub fn first_method_of_impl(
    an_impl: syn::ItemImpl,
) -> Option<syn::ImplItemMethod> {
    for item in an_impl.items {
        if let syn::ImplItem::Method(method) = item {
            return Some(method);
        }
    }
    None
}

pub fn non_self_argument_type(arg: &FnArg) -> Option<syn::Type> {
    let arg_ts_opt = match arg {
        syn::FnArg::Receiver(_) => return None,
        syn::FnArg::Typed(pt) => {
            let mut t = &pt.ty;
            t = match t.as_ref() {
                syn::Type::Reference(t) => &t.elem,
                _ => t,
            };
            Some(quote!(#t))
        }
    };
    arg_ts_opt.map(syn::Type::Verbatim)
}

pub fn non_self_argument_types(sig: &syn::Signature) -> Vec<syn::Type> {
    let mut v = Vec::new();
    for input in &sig.inputs {
        if let Some(t) = non_self_argument_type(input) {
            v.push(t);
        }
    }
    v
}

pub fn return_type_of_sig(sig: &syn::Signature) -> syn::Type {
    let ret_obj = sig.output.clone();
    let ret = match ret_obj {
        syn::ReturnType::Default => {
            quote::quote_spanned!(sig.paren_token.span=> ())
        }
        syn::ReturnType::Type(_, r) => quote!(#r),
    };
    syn::Type::Verbatim(ret)
}
