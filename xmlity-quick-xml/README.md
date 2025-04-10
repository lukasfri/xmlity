# # XMLity Quick XML &emsp; [![Build Status]][actions] [![Latest Version]][crates.io] [![Latest Docs]][docs.rs] [![xmlity msrv]][Rust 1.82]

[Build Status]: https://img.shields.io/github/actions/workflow/status/lukasfri/xmlity/rust.yaml?branch=main
[actions]: https://github.com/lukasfri/xmlity/actions?query=branch%3Amain
[Latest Version]: https://img.shields.io/crates/v/xmlity-quick-xml.svg
[crates.io]: https://crates.io/crates/xmlity-quick-xml
[Latest Docs]: https://img.shields.io/badge/docs.rs-Latest-bbbbbb.svg
[docs.rs]: https://docs.rs/xmlity-quick-xml/latest/xmlity_quick_xml
[xmlity msrv]: https://img.shields.io/badge/rustc-1.82.0+-ab6000.svg
[Rust 1.82]: https://blog.rust-lang.org/2023/06/01/Rust-1.82.0.html

This crate contains the implementation of the [`quick_xml`] backend for XMLity. It is the intention to keep this crate up to date with the latest version of `quick-xml` and `xmlity`.

## Usage

The easiest way is using the `from_str` and `to_string` functions:

```rust
use xmlity::{Serialize, Deserialize};;

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

But it is also possible to manually create the deserializer and serializer from a `quick_xml::NsReader` and `quick_xml::Writer` respectively:

```rust
use xmlity::{Serialize, Deserialize};;

#[derive(Serialize, Deserialize)]
#[xelement(name = "single_element")]
struct SingleElement(pub String);

let single_element = SingleElement("Value".to_string());

let serializer = quick_xml::writer::Writer::new(Vec::new());
let mut serializer = xmlity_quick_xml::Serializer::new(serializer);
single_element.serialize(&mut serializer).unwrap();
let bytes = serializer.into_inner();
let xml = String::from_utf8(bytes).unwrap();

assert_eq!(xml, r#"<single_element>Value</single_element>"#);

let mut ns_reader = quick_xml::NsReader::from_reader(xml.as_bytes());
let mut deserializer = xmlity_quick_xml::Deserializer::new(ns_reader);
let single_element = SingleElement::deserialize(&mut deserializer).unwrap();
assert_eq!(single_element.0, "Value");
```

Creating deserializers and serializers manually gives you more control including modifying namespace prefixes and scopes.

## License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Serde by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
</sub>
