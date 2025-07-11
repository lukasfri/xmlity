use xmlity::de::DeserializeContext;

use crate::define_deserialize_test;
use xmlity::XmlNamespace;
use xmlity::{Deserialize, Prefix};

#[derive(Debug, PartialEq)]
pub struct InternalReference(pub XmlNamespace<'static>);

impl<'de> Deserialize<'de> for InternalReference {
    fn deserialize<D: xmlity::Deserializer<'de>>(reader: D) -> Result<Self, D::Error> {
        struct __Visitor;

        impl<'de> xmlity::de::Visitor<'de> for __Visitor {
            type Value = InternalReference;

            fn visit_text<E, V>(self, value: V) -> Result<Self::Value, E>
            where
                E: xmlity::de::Error,
                V: xmlity::de::XmlText<'de>,
            {
                let prefix = Prefix::new(value.as_str()).map_err(|e| {
                    E::custom(format!("Internal reference is not valid prefix: {e:?}"))
                })?;

                let namespace = value
                    .context()
                    .resolve_prefix(prefix.as_ref())
                    .ok_or_else(|| E::custom(format!("Prefix {prefix:?} is not defined")))?
                    .into_owned();

                Ok(InternalReference(namespace))
            }

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("Internal reference")
            }
        }

        reader.deserialize_any(__Visitor)
    }
}

#[derive(Debug, PartialEq, Deserialize)]
#[xelement(
    name = "prefix",
    namespace = "http://my.namespace.example.com/this/is/a/namespace",
    preferred_prefix = "myns"
)]
pub struct PrefixElement(InternalReference);

define_deserialize_test!(
    element_with_internal_reference,
    [(PrefixElement(
       InternalReference(XmlNamespace::new_dangerous("http://my.namespace.example.com/this/is/a/namespace"))
    ), "<myns:prefix xmlns:myns=\"http://my.namespace.example.com/this/is/a/namespace\">myns</myns:prefix>")]
);

#[derive(Debug, PartialEq)]
pub struct DefaultNamespace(pub Option<XmlNamespace<'static>>);

impl<'de> Deserialize<'de> for DefaultNamespace {
    fn deserialize<D: xmlity::Deserializer<'de>>(reader: D) -> Result<Self, D::Error> {
        struct __Visitor;

        impl<'de> xmlity::de::Visitor<'de> for __Visitor {
            type Value = DefaultNamespace;

            fn visit_text<E, V>(self, value: V) -> Result<Self::Value, E>
            where
                E: xmlity::de::Error,
                V: xmlity::de::XmlText<'de>,
            {
                let namespace = value
                    .context()
                    .default_namespace()
                    .map(XmlNamespace::into_owned);

                Ok(DefaultNamespace(namespace))
            }

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("Internal reference")
            }
        }

        reader.deserialize_any(__Visitor)
    }
}

#[derive(Debug, PartialEq, Deserialize)]
#[xelement(
    name = "default",
    namespace = "http://my.namespace.example.com/this/is/a/namespace"
)]
pub struct NamespaceElement(DefaultNamespace);

define_deserialize_test!(
    element_with_default_namespace,
    [(
        NamespaceElement(DefaultNamespace(Some(XmlNamespace::new_dangerous(
            "http://my.namespace.example.com/this/is/a/namespace"
        )))),
        "<default xmlns=\"http://my.namespace.example.com/this/is/a/namespace\">Abcde</default>"
    )]
);
