//! Namespaces and Prefixes Examples
//!
//! Demonstrates XML namespaces, prefixes, and namespace expressions.

use xmlity::{Deserialize, Serialize, XmlNamespace};
use xmlity_quick_xml::{from_str, to_string};

// Simple namespace usage
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[xelement(name = "book", namespace = "http://example.com/books")]
struct Book {
    #[xattribute(name = "isbn")]
    isbn: String,

    #[xelement(name = "title")]
    title: String,

    #[xelement(name = "author")]
    author: String,
}

// Using namespace expressions
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[xelement(name = "library", namespace_expr = XmlNamespace::new_dangerous("http://example.com/library"))]
struct Library {
    #[xattribute(name = "name")]
    name: String,

    #[xelement(name = "book")]
    books: Vec<Book>,
}

// Preferred prefixes
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[xelement(
    name = "publication",
    namespace = "http://example.com/publications",
    preferred_prefix = "pub"
)]
struct Publication {
    #[xattribute(name = "id", preferred_prefix = "pub")]
    id: String,

    #[xelement(name = "title", preferred_prefix = "pub")]
    title: String,

    #[xelement(
        name = "author",
        namespace = "http://example.com/authors",
        preferred_prefix = "auth"
    )]
    author: Author,

    #[xelement(
        name = "publisher",
        namespace = "http://example.com/publishers",
        preferred_prefix = "pub-house"
    )]
    publisher: Publisher,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[xelement(name = "author", namespace = "http://example.com/authors")]
struct Author {
    #[xattribute(name = "id")]
    id: String,

    #[xelement(name = "name")]
    name: String,

    #[xelement(name = "bio", optional = true)]
    bio: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[xelement(name = "publisher", namespace = "http://example.com/publishers")]
struct Publisher {
    #[xattribute(name = "id")]
    id: String,

    #[xelement(name = "name")]
    name: String,

    #[xelement(name = "location")]
    location: String,
}

// Enforced prefixes
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[xelement(
    name = "catalog",
    namespace = "http://example.com/catalog",
    preferred_prefix = "cat"
)]
struct Catalog {
    #[xattribute(name = "version", preferred_prefix = "cat")]
    version: String,

    #[xelement(name = "section", preferred_prefix = "cat")]
    sections: Vec<Section>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[xelement(name = "section", namespace = "http://example.com/catalog")]
struct Section {
    #[xattribute(name = "name")]
    name: String,

    #[xelement(name = "item")]
    items: Vec<CatalogItem>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[xelement(name = "item", namespace = "http://example.com/catalog")]
struct CatalogItem {
    #[xattribute(name = "id")]
    id: String,

    #[xvalue]
    description: String,
}

// Mixed namespaces with attributes
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "document", namespace = "http://example.com/docs")]
struct Document {
    #[xattribute(name = "id")]
    id: String,

    #[xattribute(name = "lang", namespace = "http://www.w3.org/XML/1998/namespace")]
    language: String,

    #[xattribute(
        name = "version",
        namespace = "http://example.com/metadata",
        preferred_prefix = "meta"
    )]
    version: String,

    #[xelement(name = "title")]
    title: String,

    #[xelement(
        name = "content",
        namespace = "http://example.com/content",
        preferred_prefix = "cnt"
    )]
    content: DocumentContent,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "content", namespace = "http://example.com/content")]
struct DocumentContent {
    #[xattribute(name = "type")]
    content_type: String,

    #[xvalue]
    text: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Namespaces and Prefixes ===\n");

    // Basic book with namespaces
    let book = Book {
        isbn: "978-0-13-110362-7".to_string(),
        title: "The C Programming Language".to_string(),
        author: "Dennis Ritchie".to_string(),
    };

    let xml = to_string(&book)?;
    println!("Book:\n{}\n", xml);

    let parsed: Book = from_str(&xml)?;
    assert_eq!(book, parsed);

    // Library with namespace expression
    let library = Library {
        name: "Central Library".to_string(),
        books: vec![book.clone()],
    };

    let xml = to_string(&library)?;
    println!("Library:\n{}\n", xml);

    let parsed: Library = from_str(&xml)?;
    assert_eq!(library, parsed);

    // Publication with preferred prefixes
    let publication = Publication {
        id: "PUB-001".to_string(),
        title: "Advanced Rust Programming".to_string(),
        author: Author {
            id: "AUTH-001".to_string(),
            name: "Jane Smith".to_string(),
            bio: Some("Expert Rust developer".to_string()),
        },
        publisher: Publisher {
            id: "PUBLISHER-001".to_string(),
            name: "Tech Books Inc.".to_string(),
            location: "San Francisco".to_string(),
        },
    };

    let xml = to_string(&publication)?;
    println!("Publication:\n{}\n", xml);

    let parsed: Publication = from_str(&xml)?;
    assert_eq!(publication, parsed);

    // Catalog with enforced prefixes
    let catalog = Catalog {
        version: "2.0".to_string(),
        sections: vec![Section {
            name: "Programming Languages".to_string(),
            items: vec![
                CatalogItem {
                    id: "RUST-001".to_string(),
                    description: "Rust programming language".to_string(),
                },
                CatalogItem {
                    id: "GO-001".to_string(),
                    description: "Go programming language".to_string(),
                },
            ],
        }],
    };

    let xml = to_string(&catalog)?;
    println!("Catalog:\n{}\n", xml);

    let parsed: Catalog = from_str(&xml)?;
    assert_eq!(catalog, parsed);

    // Document with mixed namespaces
    let document = Document {
        id: "DOC-001".to_string(),
        language: "en".to_string(),
        version: "1.0".to_string(),
        title: "Sample Document".to_string(),
        content: DocumentContent {
            content_type: "text/plain".to_string(),
            text: "This is a sample document with mixed namespaces.".to_string(),
        },
    };

    let xml = to_string(&document)?;
    println!("Document:\n{}\n", xml);

    let parsed: Document = from_str(&xml)?;
    assert_eq!(document, parsed);

    println!("All examples passed!");
    Ok(())
}
