// Tool to inject .rustc_proc_macro_decls metadata into a WASM file

use std::fs;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input.wasm> <output.wasm>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];

    // Read the WASM file
    let wasm_bytes = fs::read(input_path).expect("Failed to read input WASM");

    // Parse WASM to find where to insert custom section
    // We'll need to use wasmparser or manually construct it

    // Metadata format: "derive:Demo:demo\n"
    let metadata = b"derive:Demo:demo\n";

    // Create a custom section
    // WASM custom section format:
    // - section id (0 for custom)
    // - section size (varuint32)
    // - name length (varuint32)
    // - name bytes (".rustc_proc_macro_decls")
    // - content bytes (metadata)

    let section_name = b".rustc_proc_macro_decls";

    let mut custom_section = vec![];
    custom_section.push(0); // Custom section ID

    // Calculate section size: name_len + name + content
    let name_len_encoded = encode_varuint32(section_name.len() as u32);
    let content_len = metadata.len();
    let section_size = name_len_encoded.len() + section_name.len() + content_len;
    custom_section.extend_from_slice(&encode_varuint32(section_size as u32));

    // Add name
    custom_section.extend_from_slice(&name_len_encoded);
    custom_section.extend_from_slice(section_name);

    // Add content
    custom_section.extend_from_slice(metadata);

    // Insert custom section after WASM header
    // WASM format: magic (4 bytes) + version (4 bytes) + sections...
    let mut output = vec![];
    output.extend_from_slice(&wasm_bytes[0..8]); // Copy header
    output.extend_from_slice(&custom_section); // Add custom section
    output.extend_from_slice(&wasm_bytes[8..]); // Copy rest

    fs::write(output_path, output).expect("Failed to write output WASM");
    println!("Metadata injected successfully!");
}

fn encode_varuint32(mut value: u32) -> Vec<u8> {
    let mut result = vec![];
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        result.push(byte);
        if value == 0 {
            break;
        }
    }
    result
}
