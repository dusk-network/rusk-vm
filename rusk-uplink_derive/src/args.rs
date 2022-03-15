use proc_macro2::Span;
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::{Ident, LitStr, Token};

///
/// Example usages:
///
/// `#[execute(name="read")]`
/// `#[apply(name="incr")]`
#[derive(Clone)]
pub struct Args {
    pub name: String,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name_opt: Option<String> = None;
        loop {
            let ident = input.parse::<Ident>()?;
            let _ = input.parse::<Token![=]>()?;
            let ident_str = ident.to_string();

            if ident_str.as_str() == "name" {
                name_opt = Some(input.parse::<LitStr>()?.value());
            }
            match input.parse::<Token![,]>() {
                Ok(_) => continue,
                Err(_) => break,
            }
        }
        match name_opt {
            Some(name) => Ok(Args { name }),
            None => Err(error()),
        }
    }
}

fn error() -> Error {
    let msg = r#"expected #[quote|transaction(name="<name>",buf=<size>)]"#;
    Error::new(Span::call_site(), msg)
}
