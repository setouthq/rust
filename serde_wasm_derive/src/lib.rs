extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

#[proc_macro_derive(Serialize)]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let serialization = match &input.data {
        Data::Struct(data) => {
            match &data.fields {
                Fields::Named(fields) => {
                    let field_names = fields.named.iter().map(|f| &f.ident);
                    let field_names2 = fields.named.iter().map(|f| &f.ident);
                    quote! {
                        impl #name {
                            pub fn serialize(&self) -> String {
                                let mut result = String::from("{");
                                #(
                                    result.push_str(&format!("\"{}\":{:?},", stringify!(#field_names), self.#field_names2));
                                )*
                                result.pop(); // Remove trailing comma
                                result.push('}');
                                result
                            }
                        }
                    }
                },
                Fields::Unnamed(fields) => {
                    let field_indices = (0..fields.unnamed.len()).map(syn::Index::from);
                    quote! {
                        impl #name {
                            pub fn serialize(&self) -> String {
                                let mut result = String::from("[");
                                #(
                                    result.push_str(&format!("{:?},", self.#field_indices));
                                )*
                                result.pop(); // Remove trailing comma
                                result.push(']');
                                result
                            }
                        }
                    }
                },
                Fields::Unit => {
                    quote! {
                        impl #name {
                            pub fn serialize(&self) -> String {
                                String::from("null")
                            }
                        }
                    }
                }
            }
        },
        _ => panic!("Serialize can only be derived for structs"),
    };

    TokenStream::from(serialization)
}

#[proc_macro_derive(Deserialize)]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Simplified deserialize - just creates a default instance
    let deserialization = quote! {
        impl #name {
            pub fn deserialize(s: &str) -> Self {
                // Simplified: would need a proper parser in real implementation
                Self::default()
            }
        }
    };

    TokenStream::from(deserialization)
}
