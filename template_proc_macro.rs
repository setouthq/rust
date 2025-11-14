// Minimal template proc macro for extracting metadata structure
extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro_derive(Template)]
pub fn template_derive(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
