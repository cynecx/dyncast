use proc_macro2::Span;
use syn::parse::{Error, Parse, ParseStream};

#[derive(Copy, Clone)]
pub struct Args {}

impl Args {
    fn try_parse(_input: ParseStream) -> Result<Self, Error> {
        Ok(Self {})
    }
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self, Error> {
        match Self::try_parse(input) {
            Ok(args) if input.is_empty() => Ok(args),
            _ => Err(Error::new(Span::call_site(), "unexpected arguments")),
        }
    }
}
