use syn::{
    parse::{Error, Parse, ParseStream},
    LitInt, Token,
};

mod kw {
    syn::custom_keyword!(seed);
}

#[derive(Copy, Clone)]
pub struct Args {
    pub seed: Option<u64>,
}

impl Args {
    fn try_parse(input: ParseStream) -> Result<Self, Error> {
        let mut seed = None;

        if input.peek(kw::seed) {
            input.parse::<kw::seed>()?;
            input.parse::<Token![=]>()?;
            seed = Some(input.parse::<LitInt>()?.base10_parse::<u64>()?);
        }

        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

        Ok(Self { seed })
    }
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self, Error> {
        match Self::try_parse(input) {
            Ok(args) if input.is_empty() => Ok(args),
            Ok(_) => Err(Error::new(input.span(), "unexpected arguments")),
            Err(err) => Err(err),
        }
    }
}
