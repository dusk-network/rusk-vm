use proc_macro2::Span;
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::{LitStr, Ident, Token};


#[derive(Clone)]
pub struct Args {
    pub name: String,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse::<Ident>()?;
        let _ = input.parse::<Token![=]>()?;
        let ident_str = ident.to_string();

        match ident_str.as_str() {
            "name" => {
                let name: LitStr = input.parse::<LitStr>()?;
                Ok(Args{ name: name.value() })
            }
            _ => {
                Err(error())
            }
        }
    }
}

fn error() -> Error {
    let msg = "expected #[quote|transaction(name='<name>')]";
    Error::new(Span::call_site(), msg)
}
