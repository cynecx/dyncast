use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

mod args;
mod dyncast;
// mod hash;
mod linker;
mod parse;

#[proc_macro_attribute]
pub fn dyncast(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as args::Args);
    let mut item = parse_macro_input!(input as parse::Item);
    let expanded = match dyncast::expand(&mut item, args) {
        Ok(expanded) => expanded,
        Err(err) => err.to_compile_error(),
    };
    TokenStream::from(quote! {
        #item
        #expanded
    })
}
