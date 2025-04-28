#![allow(dead_code)]

use xmlity::{Prefix, XmlNamespace};

pub fn quick_xml_serialize_test<T: xmlity::Serialize + std::fmt::Debug>(
    input: T,
) -> Result<String, xmlity_quick_xml::Error> {
    quick_xml_serialize_test_with_default(input, None)
}

pub fn quick_xml_serialize_test_with_default<T: xmlity::Serialize + std::fmt::Debug>(
    input: T,
    default_namespace: Option<XmlNamespace<'static>>,
) -> Result<String, xmlity_quick_xml::Error> {
    let serializer = quick_xml::Writer::new(Vec::new());
    let mut serializer = xmlity_quick_xml::Serializer::from(serializer);
    serializer.add_preferred_prefix(
        XmlNamespace::new("http://my.namespace.example.com/this/is/a/namespace").unwrap(),
        Prefix::new("testns").expect("testns is a valid prefix"),
    );
    if let Some(default_namespace) = default_namespace {
        serializer.add_preferred_prefix(default_namespace, Prefix::default());
    }

    input.serialize(&mut serializer)?;
    let actual_xml = String::from_utf8(serializer.into_inner()).unwrap();

    Ok(actual_xml)
}

pub fn quick_xml_deserialize_test<T: xmlity::DeserializeOwned + std::fmt::Debug>(
    input: &str,
) -> Result<T, xmlity_quick_xml::Error> {
    let reader = quick_xml::NsReader::from_reader(input.as_bytes());

    let mut deserializer = xmlity_quick_xml::de::Deserializer::from(reader);

    T::deserialize_seq(&mut deserializer)
}

pub fn clean_string(input: &str) -> String {
    input.trim().lines().map(str::trim).collect::<String>()
}

#[macro_export]
macro_rules! define_test {
    ($name: ident, [$(($xml:expr, $value:expr)),*]) =>  {
        $crate::define_test!($name, [$(($xml, $value)),*], []);
    };
    ($name: ident, [$(($xml:expr, $value:expr)),*], []) => {
        mod $name {
            #[allow(unused_imports)]
            use super::*;
            #[rstest::rstest]
            $(
                #[case($value, $xml)]
            )*
            fn serialize<T: xmlity::Serialize + std::fmt::Debug + PartialEq>(#[case] to: T, #[case] expected: &str) {
                let actual = $crate::utils::quick_xml_serialize_test(to).unwrap();

                pretty_assertions::assert_eq!(actual, expected);
            }

            #[rstest::rstest]
            $(
                #[case($value, $xml)]
            )*
            fn deserialize<T: xmlity::DeserializeOwned + std::fmt::Debug + PartialEq>(#[case] expected: T, #[case] xml: &str) {
                let actual: T = $crate::utils::quick_xml_deserialize_test(xml).unwrap();

                pretty_assertions::assert_eq!(actual, expected);
            }
        }
    };
}
