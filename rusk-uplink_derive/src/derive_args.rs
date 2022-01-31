use proc_macro2::Span;
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::{LitBool, Ident, Token};

const DERIVE_NEW_DEFAULT: bool = true;

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
            "new" => input.parse::<LitBool>().map(|a|a.value),
            _ => Err(error())
        }
    }
}

impl Parse for DeriveArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let derive_new = DeriveArgs::parse_new(input).unwrap_or(DERIVE_NEW_DEFAULT);
        Ok(DeriveArgs { derive_new })
    }
}

fn error() -> Error {
    let msg = "expected #[state|argument(new=true|false)]";
    Error::new(Span::call_site(), msg)
}
