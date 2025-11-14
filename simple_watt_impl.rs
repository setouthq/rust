// Simple watt-compatible proc macro implementation
// Uses proc_macro2 which exports the required functions

extern crate proc_macro2;

use proc_macro2::TokenStream;

#[no_mangle]
pub extern "C" fn simple_test(input: TokenStream) -> TokenStream {
    // Just return a simple fixed implementation
    // In a real macro, we'd parse the input using syn

    "impl Test { fn test() -> u32 { 42 } }".parse().unwrap()
}
