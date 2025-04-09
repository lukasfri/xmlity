# Getting started

1. Add `xmlity` dependency to your `Cargo.toml` file:

```toml
[dependencies]
xmlity = { version = "0.1", features = ["derive"] }

# You can use any XML library you want, but the officially supported one is `quick-xml` using the `xmlity-quick-xml` crate.
xmlity-quick-xml = "0.1"
```

2. Define your data model:

```rust
extern crate xmlity;
extern crate xmlity_derive;
extern crate xmlity_quick_xml;

use xmlity::{Serialize, Deserialize};;
use xmlity_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
#[xelement(name = "name")]
struct Name(String);

#[derive(Serialize, Deserialize)]
#[xelement(name = "age")]
struct Age(u8);

#[derive(Serialize, Deserialize)]
#[xelement(name = "person")]
struct Person {
    name: Name,
    age: Age,
}

let person = Person {
    name: Name("John".to_string()),
    age: Age(42),
};

let xml = xmlity_quick_xml::to_string(&person).expect("Failed to serialize");
assert_eq!(xml, r#"<person><name>John</name><age>42</age></person>"#);

let person: Person = xmlity_quick_xml::from_str(&xml).expect("Failed to deserialize");
assert_eq!(person.name.0, "John");
assert_eq!(person.age.0, 42);
```
