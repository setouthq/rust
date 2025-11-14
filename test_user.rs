// Test using the simple proc macro
extern crate simple_test_macro;

use simple_test_macro::SimpleTest;

#[derive(SimpleTest)]
struct MyStruct;

fn main() {
    println!("Test: {}", MyStruct::test());
}
