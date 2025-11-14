extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro_derive(ExamineMe)]
pub fn examine_me(input: TokenStream) -> TokenStream {
    input
}
