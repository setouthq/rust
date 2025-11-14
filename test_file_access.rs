use std::path::Path;

fn main() {
    let test_paths = [
        "simple_test_macro_new.wasm",
        "./simple_test_macro_new.wasm",
        "test_user.rs",
    ];

    for path_str in &test_paths {
        let path = Path::new(path_str);
        println!("{}: exists={}, is_file={}", path_str, path.exists(), path.is_file());

        if let Ok(metadata) = path.metadata() {
            println!("  metadata: len={}, is_file={}", metadata.len(), metadata.is_file());
        }
    }
}
