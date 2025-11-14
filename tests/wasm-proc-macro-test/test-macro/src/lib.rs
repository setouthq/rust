use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, ItemFn};

/// A simple derive macro that adds a `generated_value()` method
#[proc_macro_derive(TestDerive)]
pub fn test_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl #name {
            pub fn generated_value() -> u32 {
                42
            }
        }
    };

    TokenStream::from(expanded)
}

/// A simple attribute macro that adds a comment
#[proc_macro_attribute]
pub fn test_attr(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let name = &input.sig.ident;
    let block = &input.block;
    let vis = &input.vis;
    let sig = &input.sig;

    let expanded = quote! {
        #[doc = "Modified by test_attr"]
        #vis #sig {
            println!("Function {} called", stringify!(#name));
            #block
        }
    };

    TokenStream::from(expanded)
}

/// A simple function-like macro that generates a constant
#[proc_macro]
pub fn test_bang(_input: TokenStream) -> TokenStream {
    let expanded = quote! {
        42u32
    };

    TokenStream::from(expanded)
}

/// A derive macro with helper attributes
#[proc_macro_derive(TestDeriveWithAttrs, attributes(test_helper))]
pub fn test_derive_with_attrs(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl #name {
            pub fn with_attrs() -> &'static str {
                "helper attributes supported"
            }
        }
    };

    TokenStream::from(expanded)
}
