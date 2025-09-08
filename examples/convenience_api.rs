use postbag::{from_full_slice, from_slim_slice, to_full_vec, to_slim_vec};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Person {
    name: String,
    age: u32,
    active: bool,
}

fn main() {
    let person = Person { name: "Alice".to_string(), age: 30, active: true };

    // Test the new convenience APIs
    println!("=== Testing new convenience APIs ===");

    // Serialize to Vec<u8> using Full configuration
    let full_bytes = to_full_vec(&person).unwrap();
    println!("Full serialization: {} bytes", full_bytes.len());

    // Serialize to Vec<u8> using Slim configuration
    let slim_bytes = to_slim_vec(&person).unwrap();
    println!("Slim serialization: {} bytes", slim_bytes.len());

    // Deserialize from slice using Full configuration
    let person_from_full: Person = from_full_slice(&full_bytes).unwrap();
    println!("Deserialized from full: {:?}", person_from_full);
    assert_eq!(person, person_from_full);

    // Deserialize from slice using Slim configuration
    let person_from_slim: Person = from_slim_slice(&slim_bytes).unwrap();
    println!("Deserialized from slim: {:?}", person_from_slim);
    assert_eq!(person, person_from_slim);

    println!("✓ All new convenience functions work correctly!");
    println!("✓ Space saved with slim: {} bytes", full_bytes.len() - slim_bytes.len());
}
