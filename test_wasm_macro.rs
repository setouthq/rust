// Test file for WASM proc macro
// Should work with: --wasm-proc-macro Demo=watt_demo_with_metadata.wasm

#[derive(Demo)]
struct TestStruct {
    field: i32,
}

fn main() {
    println!("Test");
}
