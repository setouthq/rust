#[derive(Demo)]
struct MyData;

fn main() {
    println!("{}", MyData::MESSAGE);
    
    // Manually demonstrate serialize-like functionality
    let data = r#"{"type":"MyData","values":[1,2,3]}"#;
    println!("Serialized data: {}", data);
}
