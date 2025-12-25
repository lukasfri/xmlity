#![allow(dead_code)]

use std::{collections::BTreeMap, str::FromStr};

use xmlity::{PrefixBuf, XmlNamespaceBuf};

pub fn quick_xml_serialize_test<T: xmlity::Serialize + std::fmt::Debug>(
    input: T,
) -> Result<String, xmlity_quick_xml::ser::Error> {
    quick_xml_serialize_test_with_default(input, None)
}

pub fn quick_xml_serialize_test_with_default<T: xmlity::Serialize + std::fmt::Debug>(
    input: T,
    default_namespace: Option<XmlNamespaceBuf>,
) -> Result<String, xmlity_quick_xml::ser::Error> {
    let serializer = quick_xml::Writer::new(Vec::new());
    let mut serializer = xmlity_quick_xml::Serializer::new_with_namespaces(serializer, {
        let mut map = BTreeMap::new();
        map.insert(
            XmlNamespaceBuf::from_str("http://my.namespace.example.com/this/is/a/namespace")
                .unwrap(),
            PrefixBuf::from_str("testns").expect("testns is a valid prefix"),
        );
        if let Some(default_namespace) = default_namespace {
            map.insert(default_namespace, PrefixBuf::default());
        }
        map
    });

    input.serialize(&mut serializer)?;
    let actual_xml = String::from_utf8(serializer.into_inner()).unwrap();

    Ok(actual_xml)
}

pub fn quick_xml_deserialize_test<T: xmlity::DeserializeOwned + std::fmt::Debug>(
    input: &str,
) -> Result<T, xmlity_quick_xml::de::Error> {
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
            $crate::define_serialize_test!(serialize, [$(($value, $serialize_xml)),*]);


            $crate::define_deserialize_test!(deserialize, [$(($value, $deserialize_xml)),*]);
        }
    };
    (@internal $name: ident, [$($existing:tt)*], [($value:expr, $serialize_xml2:expr, $deserialize_xml2:expr)]) => {
        $crate::define_test!(@impl $name, [$($existing)*, ($value, $serialize_xml2, $deserialize_xml2)]);
    };
    (@internal $name: ident, [$($existing:tt)*], [($value:expr, $xml:expr)]) => {
        $crate::define_test!(@impl $name, [$($existing)*, ($value, $xml, $xml)]);
    };
    (@internal $name: ident, [$($existing:tt)*], [($value:expr, $serialize_xml2:expr, $deserialize_xml2:expr), $($tail:tt)*]) => {
        $crate::define_test!(@internal $name, [$($existing)*, ($value, $serialize_xml2, $deserialize_xml2)], [$($tail)*]);
    };
    (@internal $name: ident, [$($existing:tt)*], [($value:expr, $xml:expr), $($tail:tt)*]) => {
        $crate::define_test!(@internal $name, [$($existing)*, ($value, $xml, $xml)], [$($tail)*]);
    };
    ($name: ident, [$($tail:tt)*]) => {
        $crate::define_test!(@internal $name, [], [$($tail)*]);
    };
    // ($name: ident, []) => {
    //     $crate::define_test!($name, [], []);
    // };
}

#[macro_export]
macro_rules! define_serialize_test {
    // Main implementation of the macro
    (@impl $name: ident, [$(,)*$(($value:expr, $xml:expr)),*]) => {
        #[allow(unused_imports)]
        use super::*;
        #[rstest::rstest]
        $(
            #[case($value, $xml)]
        )*
        #[ntest::timeout(1000)]
        fn $name<T: xmlity::Serialize + std::fmt::Debug + PartialEq, U: AsRef<str>>(#[case] to: T, #[case] expected: U) {
            let actual = $crate::utils::quick_xml_serialize_test(to).unwrap();

            pretty_assertions::assert_eq!(actual, expected.as_ref());
        }
    };
    (@internal $name: ident, [$($existing:tt)*], [($value:expr, $xml:expr)]) => {
        $crate::define_serialize_test!(@impl $name, [$($existing)*, ($value, $xml)]);
    };
    (@internal $name: ident, [$($existing:tt)*], [($value:expr, $xml:expr), $($tail:tt)*]) => {
        $crate::define_serialize_test!(@internal $name, [$($existing)*, ($value,$xml)], [$($tail)*]);
    };
    ($name: ident, [$($tail:tt)*]) => {
        $crate::define_serialize_test!(@internal $name, [], [$($tail)*]);
    };
}

#[macro_export]
macro_rules! define_deserialize_test {
    // Main implementation of the macro
    (@impl $name: ident, [$(,)*$(($value:expr, $xml:expr)),*]) => {
        #[rstest::rstest]
        $(
            #[case($value, $xml)]
        )*
        #[ntest::timeout(1000)]
        fn $name<T: xmlity::DeserializeOwned + std::fmt::Debug + PartialEq, U: AsRef<str>>(#[case] expected: T, #[case] xml: U) {
            let actual: T = $crate::utils::quick_xml_deserialize_test(xml.as_ref()).unwrap();

            pretty_assertions::assert_eq!(actual, expected);
        }
    };
    (@internal $name: ident, [$($existing:tt)*], [($value:expr, $xml:expr)]) => {
        $crate::define_deserialize_test!(@impl $name, [$($existing)*, ($value, $xml)]);
    };
    (@internal $name: ident, [$($existing:tt)*], [($value:expr, $xml:expr), $($tail:tt)*]) => {
        $crate::define_deserialize_test!(@internal $name, [$($existing)*, ($value,$xml)], [$($tail)*]);
    };
    ($name: ident, [$($tail:tt)*]) => {
        $crate::define_deserialize_test!(@internal $name, [], [$($tail)*]);
    };
}
