#[derive(Demo)]
struct User;

#[derive(Demo)]
struct Product;

// Manual serialization to demonstrate serde-like patterns
struct UserData {
    id: u32,
    name: String,
    email: String,
}

impl UserData {
    fn new(id: u32, name: String, email: String) -> Self {
        UserData { id, name, email }
    }
    
    fn serialize(&self) -> String {
        format!(
            r#"{{"id":{},"name":"{}","email":"{}"}}"#,
            self.id, self.name, self.email
        )
    }
}

struct ProductData {
    id: u32,
    name: String,
    price: f64,
}

impl ProductData {
    fn new(id: u32, name: String, price: f64) -> Self {
        ProductData { id, name, price }
    }
    
    fn serialize(&self) -> String {
        format!(
            r#"{{"id":{},"name":"{}","price":{}}}"#,
            self.id, self.name, self.price
        )
    }
}

fn main() {
    // Demonstrate proc macro generated constants
    println!("=== Proc Macro Generated Messages ===");
    println!("User: {}", User::MESSAGE);
    println!("Product: {}", Product::MESSAGE);
    
    // Demonstrate serde-like serialization
    println!("\n=== Serialization Demo (Serde-like) ===");
    
    let user = UserData::new(1, "Alice".to_string(), "alice@example.com".to_string());
    println!("User JSON: {}", user.serialize());
    
    let product = ProductData::new(101, "Laptop".to_string(), 999.99);
    println!("Product JSON: {}", product.serialize());
    
    // Simulate array serialization
    println!("\n=== Array Serialization ===");
    let users_json = format!(
        "[{},{}]",
        UserData::new(1, "Alice".to_string(), "alice@example.com".to_string()).serialize(),
        UserData::new(2, "Bob".to_string(), "bob@example.com".to_string()).serialize()
    );
    println!("Users: {}", users_json);
}
