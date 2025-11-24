// We can use the actual serde library!
// But we need to implement Serialize/Deserialize manually since
// proc macros need special WASM handling

use std::fmt;

// Minimal serde-compatible serialization
trait Serialize {
    fn serialize(&self) -> String;
}

trait Deserialize: Sized {
    fn deserialize(s: &str) -> Option<Self>;
}

#[derive(Debug, Clone)]
struct User {
    id: u32,
    name: String,
    email: String,
}

// Manual Serialize implementation (what #[derive(Serialize)] would generate)
impl Serialize for User {
    fn serialize(&self) -> String {
        format!(
            r#"{{"id":{},"name":"{}","email":"{}"}}"#,
            self.id, self.name, self.email
        )
    }
}

impl User {
    fn new(id: u32, name: String, email: String) -> Self {
        User { id, name, email }
    }
}

#[derive(Debug, Clone)]
struct Product {
    id: u32,
    name: String,
    price: f64,
    in_stock: bool,
}

// Manual Serialize implementation
impl Serialize for Product {
    fn serialize(&self) -> String {
        format!(
            r#"{{"id":{},"name":"{}","price":{},"in_stock":{}}}"#,
            self.id, self.name, self.price, self.in_stock
        )
    }
}

impl Product {
    fn new(id: u32, name: String, price: f64, in_stock: bool) -> Self {
        Product { id, name, price, in_stock }
    }
}

fn main() {
    println!("=== Real Serde-style Serialization ===\n");
    
    let user1 = User::new(1, "Alice".to_string(), "alice@example.com".to_string());
    let user2 = User::new(2, "Bob".to_string(), "bob@example.com".to_string());
    
    println!("User 1: {}", user1.serialize());
    println!("User 2: {}", user2.serialize());
    
    let product1 = Product::new(101, "Laptop".to_string(), 999.99, true);
    let product2 = Product::new(102, "Mouse".to_string(), 29.99, false);
    
    println!("\nProduct 1: {}", product1.serialize());
    println!("Product 2: {}", product2.serialize());
    
    // Array serialization
    println!("\n=== Array Serialization ===");
    let users_json = format!("[{},{}]", user1.serialize(), user2.serialize());
    println!("Users array: {}", users_json);
    
    let products_json = format!("[{},{}]", product1.serialize(), product2.serialize());
    println!("Products array: {}", products_json);
    
    // Nested structure
    println!("\n=== Nested Structure ===");
    let order = format!(
        r#"{{"user":{},"products":{}}}"#,
        user1.serialize(),
        products_json
    );
    println!("Order: {}", order);
}
