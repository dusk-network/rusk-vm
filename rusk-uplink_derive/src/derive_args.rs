// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use proc_macro2::Span;
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::{Ident, LitBool, Token};

const DERIVE_NEW_DEFAULT: bool = true;

/// NOTE: `new` is optional, if missing - true is assumed
/// `new=true` causes the `new` method to be derived
/// `new=false` will stop the `new` method from being derived
///
/// Example usages:
///
/// `#[state]`
/// `#[query]`
/// `#[state(new=false)]`
/// `#[argument(new=false)]`
#[derive(Clone)]
pub struct DeriveArgs {
    pub derive_new: bool,
}

impl DeriveArgs {
    fn parse_new(input: ParseStream) -> Result<bool> {
        let ident = input.parse::<Ident>()?;
        let _ = input.parse::<Token![=]>()?;
        let ident_str = ident.to_string();
        match ident_str.as_str() {
            "new" => input.parse::<LitBool>().map(|a| a.value),
            _ => Err(error()),
        }
    }
}

impl Parse for DeriveArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let derive_new =
            DeriveArgs::parse_new(input).unwrap_or(DERIVE_NEW_DEFAULT);
        Ok(DeriveArgs { derive_new })
    }
}

fn error() -> Error {
    let msg = "expected #[state|argument(new=true|false)]";
    Error::new(Span::call_site(), msg)
}
