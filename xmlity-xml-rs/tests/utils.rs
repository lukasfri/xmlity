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
    // Main implementation of the macro
    (@impl $name: ident, [$(,)*$(($value:expr, $serialize_xml:expr, $deserialize_xml:expr)),*]) => {
        mod $name {
            #[allow(unused_imports)]
            use super::*;
            #[rstest::rstest]
            $(
                #[case($value, $serialize_xml)]
            )*
            fn serialize<T: xmlity::Serialize + std::fmt::Debug + PartialEq, U: AsRef<str>>(#[case] to: T, #[case] expected: U) {
                let actual = $crate::utils::quick_xml_serialize_test(to).unwrap();

                pretty_assertions::assert_eq!(actual, expected.as_ref());
            }

            #[rstest::rstest]
            $(
                #[case($value, $deserialize_xml)]
            )*
            fn deserialize<T: xmlity::DeserializeOwned + std::fmt::Debug + PartialEq, U: AsRef<str>>(#[case] expected: T, #[case] xml: U) {
                let actual: T = $crate::utils::quick_xml_deserialize_test(xml.as_ref()).unwrap();

                pretty_assertions::assert_eq!(actual, expected);
            }
        }
    };
    (@internal $name: ident, [$($existing:tt)*], [($value2:expr, $serialize_xml2:expr, $deserialize_xml2:expr)]) => {
        $crate::define_test!(@impl $name, [$($existing)*, ($value2, $serialize_xml2, $deserialize_xml2)]);
    };
    (@internal $name: ident, [$($existing:tt)*], [($value2:expr, $xml:expr)]) => {
        $crate::define_test!(@impl $name, [$($existing)*, ($value2, $xml, $xml)]);
    };
    (@internal $name: ident, [$($existing:tt)*], [($value2:expr, $serialize_xml2:expr, $deserialize_xml2:expr), $($tail:tt)*]) => {
        $crate::define_test!(@internal $name, [$($existing)*, ($value2, $serialize_xml2, $deserialize_xml2)], [$($tail)*]);
    };
    (@internal $name: ident, [$($existing:tt)*], [($value2:expr, $xml:expr), $($tail:tt)*]) => {
        $crate::define_test!(@internal $name, [$($existing)*, ($value2, $xml, $xml)], [$($tail)*]);
    };
    ($name: ident, [$($tail:tt)*]) => {
        $crate::define_test!(@internal $name, [], [$($tail)*]);
    };
    // ($name: ident, []) => {
    //     $crate::define_test!($name, [], []);
    // };
}
