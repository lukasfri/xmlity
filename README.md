# XMLity &emsp; [![Build Status]][actions] [![Latest Version]][crates.io] [![Latest Docs]][docs.rs] [![xmlity msrv]][Rust 1.82]

[Build Status]: https://img.shields.io/github/actions/workflow/status/lukasfri/xmlity/rust.yaml?branch=main
[actions]: https://github.com/lukasfri/xmlity/actions?query=branch%3Amain
[Latest Version]: https://img.shields.io/crates/v/xmlity.svg
[crates.io]: https://crates.io/crates/xmlity
[Latest Docs]: https://img.shields.io/badge/docs.rs-Latest-bbbbbb.svg
[docs.rs]: https://docs.rs/xmlity/latest/xmlity/
[xmlity msrv]: https://img.shields.io/badge/rustc-1.82.0+-ab6000.svg
[Rust 1.82]: https://blog.rust-lang.org/2023/06/01/Rust-1.82.0.html

XMLity is a (de)serialization library for XML, inspired by [Serde](https://serde.rs/) and improves upon XML (de)serialization libraries such as [yaserde](https://github.com/media-io/yaserde) and [quick-xml](https://github.com/tafia/quick-xml) by providing a more flexible API that is more powerful, utilising primairly a trial and error approach to parsing XML. This can inherently be a bit slower than other libraries, but it allows for more complex XML structures to be parsed.

---

## Get started

To get started, we recommend you check out the [XMLity book](https://xmlity.lukasfri.com) and [the documentation][docs.rs].

## Example

1. Add XMLity and XMLity-compatible (de)serializer library. In this example we use `xmlity_quick_xml`.

```toml
[dependencies]

xmlity = { version = "0.0.0", features = ["derive"] }

xmlity_quick_xml = "0.0.0"
```

2. Write defintions and use:

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
