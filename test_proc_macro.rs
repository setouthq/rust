// Simple test proc macro for WASM
extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro_derive(HelloMacro)]
pub fn hello_macro_derive(input: TokenStream) -> TokenStream {
    // Just generate a simple impl
    let input_str = input.to_string();

    // Extract the struct name (very simple parsing)
    let name = if let Some(start) = input_str.find("struct") {
        let after_struct = &input_str[start + 6..].trim_start();
        if let Some(end) = after_struct.find(|c: char| c.is_whitespace() || c == '{') {
            after_struct[..end].trim()
        } else {
            "Unknown"
        }
    } else {
        "Unknown"
    };

    let output = format!(
        "impl {} {{
            pub fn hello() {{
                println!(\"Hello from {}!\");
            }}
        }}",
        name, name
    );

    output.parse().unwrap()
}
