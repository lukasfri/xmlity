use xmlity::{types::utils::XmlRoot, Deserialize};

const EXAMPLE_XML_WITH_DECL: &str = r#"
<?xml version="1.0" encoding="UTF-8"?>
<root>
    <value>Example</value>
</root>
"#;

#[derive(Debug, Deserialize, PartialEq)]
#[xelement(name = "root")]
struct Root {
    #[xelement(name = "value")]
    value: String,
}

pub fn main() {
    let root = xmlity_quick_xml::from_str::<XmlRoot<Root>>(EXAMPLE_XML_WITH_DECL.trim())
        .unwrap()
        .into_value()
        .unwrap();

    assert_eq!(
        root,
        Root {
            value: "Example".to_string(),
        }
    );

    println!("Deserialized successfully: {:?}", root);
}
