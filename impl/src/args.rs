use proc_macro2::Span;
use syn::{
    parse::{Error, Parse, ParseStream},
    LitInt, LitStr, Token,
};

mod kw {
    syn::custom_keyword!(global_id);
    syn::custom_keyword!(seed);
}

#[derive(Debug, Clone)]
pub struct Args {
    pub global_id: Option<String>,
    pub seed: Option<u64>,
    pub end_span: Span,
}

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
        let mut global_id = None;
        let mut seed = None;

        parse_arg_list(input, |input| {
            if input.peek(kw::seed) {
                input.parse::<kw::seed>()?;
                input.parse::<Token![=]>()?;

                if seed.is_some() {
                    return Err(Error::new(input.span(), "seed has already been set"));
                }

                seed = Some(input.parse::<LitInt>()?.base10_parse::<u64>()?);
            }

            if input.peek(kw::global_id) {
                input.parse::<kw::global_id>()?;
                input.parse::<Token![=]>()?;

                if global_id.is_some() {
                    return Err(Error::new(input.span(), "global_id has already been set"));
                }

                let lit_str = input.parse::<LitStr>()?;
                let span = lit_str.span();
                let val = lit_str.value();

                let is_valid_id = val.chars().all(|c| match c {
                    '_' | ':' => true,
                    c if c.is_ascii_alphanumeric() => true,
                    _ => false,
                });

                if !is_valid_id {
                    return Err(Error::new(
                        span,
                        "global_id must only contain alphanumeric, '-' or ':' characters",
                    ));
                }

                global_id = Some(val);
            }

            Ok(())
        })?;

        Ok(Self {
            global_id,
            seed,
            end_span: input.span(),
        })
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
