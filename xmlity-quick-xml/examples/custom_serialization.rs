//! Custom Serialization Examples
//!
//! Demonstrates custom serialization and deserialization with:
//! - Custom date formatting
//! - Comma-separated lists
//! - Case transformations

use xmlity::{Deserialize, Deserializer, Serialize, Serializer};
use xmlity_quick_xml::{from_str, to_string};

// Custom date type with custom serialization
#[derive(Debug, Clone, PartialEq)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl std::str::FromStr for Date {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 3 {
            return Err("Invalid date format".to_string());
        }

        let year = parts[0].parse().map_err(|_| "Invalid year")?;
        let month = parts[1].parse().map_err(|_| "Invalid month")?;
        let day = parts[2].parse().map_err(|_| "Invalid day")?;

        Ok(Date { year, month, day })
    }
}

fn serialize_date<S: Serializer>(date: &CustomDate, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_text(date.0.to_string())
}

fn deserialize_date<'de, D: Deserializer<'de>>(deserializer: D) -> Result<CustomDate, D::Error> {
    let s = String::deserialize(deserializer)?;
    let date = s.parse().map_err(xmlity::de::Error::custom)?;
    Ok(CustomDate(date))
}

// Date wrapper using custom serialization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[xvalue(
    serialize_with = "serialize_date",
    deserialize_with = "deserialize_date"
)]
pub struct CustomDate(Date);

// Comma-separated list wrapper
#[derive(Debug, Clone, PartialEq)]
pub struct CommaSeparatedList(Vec<String>);

fn serialize_comma_list<S: Serializer>(list: &TagList, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_text(list.0 .0.join(","))
}

fn deserialize_comma_list<'de, D: Deserializer<'de>>(deserializer: D) -> Result<TagList, D::Error> {
    let s = String::deserialize(deserializer)?;
    Ok(TagList(CommaSeparatedList(
        s.split(',').map(|s| s.trim().to_string()).collect(),
    )))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[xvalue(
    serialize_with = "serialize_comma_list",
    deserialize_with = "deserialize_comma_list"
)]
pub struct TagList(CommaSeparatedList);

// Uppercase string wrapper
#[derive(Debug, Clone, PartialEq)]
pub struct UppercaseString(String);

fn serialize_uppercase<S: Serializer>(
    s: &CaseTransformString,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_text(s.0 .0.to_uppercase())
}

fn deserialize_lowercase<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<CaseTransformString, D::Error> {
    let s = String::deserialize(deserializer)?;
    Ok(CaseTransformString(UppercaseString(s.to_lowercase())))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[xvalue(
    serialize_with = "serialize_uppercase",
    deserialize_with = "deserialize_lowercase"
)]
pub struct CaseTransformString(UppercaseString);

// Event struct using custom date
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "event")]
struct Event {
    #[xattribute(name = "id")]
    id: String,

    #[xelement(name = "title")]
    title: String,

    #[xelement(name = "date")]
    date: CustomDate,

    #[xelement(name = "description")]
    description: String,
}

// Product struct using custom tag list
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "product")]
struct Product {
    #[xattribute(name = "sku")]
    sku: String,

    #[xelement(name = "name")]
    name: String,

    #[xelement(name = "tags")]
    tags: TagList,

    #[xelement(name = "price")]
    price: f64,
}

// Message struct using case transform
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "message")]
struct Message {
    #[xattribute(name = "id")]
    id: String,

    #[xelement(name = "sender")]
    sender: CaseTransformString,

    #[xelement(name = "content")]
    content: String,
}

// Blog post combining multiple custom types
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "blog-post")]
struct BlogPost {
    #[xattribute(name = "id")]
    id: String,

    #[xelement(name = "title")]
    title: String,

    #[xelement(name = "author")]
    author: CaseTransformString,

    #[xelement(name = "published")]
    published: CustomDate,

    #[xelement(name = "tags")]
    tags: TagList,

    #[xvalue] // Text content of the blog post
    content: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Custom Serialization ===\n");

    // Event with custom date serialization
    let event = Event {
        id: "EVT001".to_string(),
        title: "Rust Conference 2024".to_string(),
        date: CustomDate(Date {
            year: 2024,
            month: 6,
            day: 15,
        }),
        description: "Annual Rust programming conference".to_string(),
    };

    let xml = to_string(&event)?;
    println!("Event:\n{}\n", xml);

    let parsed: Event = from_str(&xml)?;
    assert_eq!(event, parsed);

    // Product with comma-separated tags
    let product = Product {
        sku: "LAPTOP001".to_string(),
        name: "Gaming Laptop".to_string(),
        tags: TagList(CommaSeparatedList(vec![
            "electronics".to_string(),
            "computers".to_string(),
            "gaming".to_string(),
            "high-performance".to_string(),
        ])),
        price: 1299.99,
    };

    let xml = to_string(&product)?;
    println!("Product:\n{}\n", xml);

    let parsed: Product = from_str(&xml)?;
    assert_eq!(product, parsed);

    // Message with case transformation
    let message = Message {
        id: "MSG001".to_string(),
        sender: CaseTransformString(UppercaseString("john_doe".to_string())),
        content: "Hello, world!".to_string(),
    };

    let xml = to_string(&message)?;
    println!("Message:\n{}\n", xml);

    let parsed: Message = from_str(&xml)?;
    assert_eq!(message, parsed);

    // Blog post with multiple custom serializations
    let blog_post = BlogPost {
        id: "POST001".to_string(),
        title: "Getting Started with XML Serialization in Rust".to_string(),
        author: CaseTransformString(UppercaseString("jane smith".to_string())),
        published: CustomDate(Date {
            year: 2024,
            month: 1,
            day: 15,
        }),
        tags: TagList(CommaSeparatedList(vec![
            "rust".to_string(),
            "xml".to_string(),
            "serialization".to_string(),
            "tutorial".to_string(),
        ])),
        content: "In this post, we'll explore how to work with XML serialization...".to_string(),
    };

    let xml = to_string(&blog_post)?;
    println!("Blog Post:\n{}\n", xml);

    let parsed: BlogPost = from_str(&xml)?;
    assert_eq!(blog_post, parsed);

    println!("All examples passed!");
    Ok(())
}
