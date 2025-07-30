//! Basic Element Examples
//!
//! Demonstrates fundamental xmlity usage:
//! - Basic struct serialization/deserialization
//! - Element naming and namespaces
//! - Text content with #[xvalue]

use xmlity::{Deserialize, Serialize};
use xmlity_quick_xml::{from_str, to_string};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "person")]
struct Person {
    #[xelement(name = "name")]
    name: String,
    #[xelement(name = "age")]
    age: u32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "book", namespace = "http://example.com/books")]
struct Book {
    #[xelement(name = "title")]
    title: String,
    #[xelement(name = "isbn")]
    isbn: String,
    #[xvalue]
    description: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "note")]
struct Note {
    #[xvalue]
    content: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Basic Elements ===\n");

    // Basic struct with named elements
    let person = Person {
        name: "Alice".to_string(),
        age: 30,
    };

    let xml = to_string(&person)?;
    println!("Person:\n{}\n", xml);

    let parsed: Person = from_str(&xml)?;
    assert_eq!(person, parsed);

    // Element with namespace
    let book = Book {
        title: "The Rust Programming Language".to_string(),
        isbn: "978-1-59327-828-1".to_string(),
        description: "A comprehensive guide to Rust programming.".to_string(),
    };

    let xml = to_string(&book)?;
    println!("Book:\n{}\n", xml);

    let parsed: Book = from_str(&xml)?;
    assert_eq!(book, parsed);

    // Simple text content
    let note = Note {
        content: "This is a simple note".to_string(),
    };

    let xml = to_string(&note)?;
    println!("Note:\n{}\n", xml);

    let parsed: Note = from_str(&xml)?;
    assert_eq!(note, parsed);

    println!("All examples passed!");
    Ok(())
}
