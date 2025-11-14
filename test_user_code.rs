// Test code that uses our WASM proc macro
#[derive(HelloMacro)]
struct MyStruct {
    field: i32,
}

fn main() {
    MyStruct::hello();
    println!("Test complete!");
}
