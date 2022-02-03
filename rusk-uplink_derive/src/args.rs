use proc_macro2::Span;
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::{Ident, LitInt, LitStr, Token};

const BUF_DEFAULT: usize = 512;

/// NOTE: `buf` is optional while `name` is not
///
/// Example usages:
///
/// `#[query(name="peek", buf=65536)]`
/// `#[query(name="read")]`
/// `#[transaction(name="pop", buf=65536)]`
/// `#[transaction(name="incr")]`
#[derive(Clone)]
pub struct Args {
    pub name: String,
    pub buf: usize,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name_opt: Option<String> = None;
        let mut buf = BUF_DEFAULT;
        loop {
            let ident = input.parse::<Ident>()?;
            let _ = input.parse::<Token![=]>()?;
            let ident_str = ident.to_string();

            match ident_str.as_str() {
                "name" => {
                    name_opt = Some(input.parse::<LitStr>()?.value());
                }
                "buf" => {
                    buf = input.parse::<LitInt>()?.base10_parse::<u32>()?
                        as usize;
                }
                _ => (),
            }
            match input.parse::<Token![,]>() {
                Ok(_) => continue,
                Err(_) => break,
            }
        }
        match name_opt {
            Some(name) => Ok(Args { name, buf }),
            None => Err(error()),
        }
    }
}

fn error() -> Error {
    let msg = r#"expected #[quote|transaction(name="<name>",buf=<size>)]"#;
    Error::new(Span::call_site(), msg)
}
