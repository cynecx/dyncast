use syn::{
    parse::{Error, Parse, ParseStream},
    Token,
};

#[derive(Debug, Clone)]
pub struct Args {}

fn parse_arg_list(
    input: ParseStream,
    mut f: impl FnMut(ParseStream) -> Result<(), Error>,
) -> Result<(), Error> {
    loop {
        if input.is_empty() {
            return Ok(());
        }

        f(input)?;

        if input.is_empty() {
            return Ok(());
        }

        let _ = input.parse::<Token![,]>()?;
    }
}

impl Args {
    fn try_parse(input: ParseStream) -> Result<Self, Error> {
        parse_arg_list(input, |input| {
            Err(Error::new(input.span(), "unexpected argument"))
        })?;

        Ok(Self {})
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
