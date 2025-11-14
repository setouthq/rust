// A minimal proc macro that doesn't need external dependencies
extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro_derive(SimpleTest)]
pub fn simple_test(_input: TokenStream) -> TokenStream {
    "impl Test { fn test() -> u32 { 42 } }".parse().unwrap()
}

#[proc_macro_attribute]
pub fn simple_attr(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro]
pub fn simple_bang(_input: TokenStream) -> TokenStream {
    "42u32".parse().unwrap()
}
