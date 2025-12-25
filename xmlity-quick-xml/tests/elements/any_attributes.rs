use std::str::FromStr;
use xmlity::{
    de::DeserializationGroupBuilder,
    value::{XmlAttribute, XmlText},
    DeserializationGroup, Deserialize, ExpandedNameBuf, LocalNameBuf, SerializationGroup,
    Serialize,
};

use crate::define_test;

#[derive(Debug, PartialEq)]
pub struct AnyAttributes {
    /// The attributes of the element.
    pub attributes: Vec<XmlAttribute>,
}

impl<'de> DeserializationGroupBuilder<'de> for AnyAttributes {
    type Value = Self;

    fn contribute_attributes<D: xmlity::de::AttributesAccess<'de>>(
        &mut self,
        mut access: D,
    ) -> Result<bool, D::Error> {
        if let Ok(Some(attr)) = access.next_attribute::<XmlAttribute>() {
            self.attributes.push(attr);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn attributes_done(&self) -> bool {
        false
    }

    fn finish<E: xmlity::de::Error>(self) -> Result<Self::Value, E> {
        Ok(self)
    }
}

impl<'de> DeserializationGroup<'de> for AnyAttributes {
    type Builder = Self;

    fn builder() -> Self::Builder {
        Self {
            attributes: Vec::new(),
        }
    }
}

impl SerializationGroup for AnyAttributes {
    fn serialize_attributes<S: xmlity::ser::SerializeAttributes>(
        &self,
        serializer: &mut S,
    ) -> Result<(), S::Error> {
        self.attributes
            .iter()
            .try_for_each(|attr| serializer.serialize_attribute(attr).map(|_| ()))
    }

    fn serialize_children<S: xmlity::ser::SerializeSeq>(
        &self,
        serializer: &mut S,
    ) -> Result<(), S::Error> {
        let _ = serializer;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "a")]
pub struct A {
    #[xgroup]
    any_attributes: AnyAttributes,
}

define_test!(
    any_attributes_element,
    [(
        A {
            any_attributes: AnyAttributes {
                attributes: vec![XmlAttribute::new(
                    ExpandedNameBuf::new(LocalNameBuf::from_str("test").unwrap(), None),
                    XmlText::new("testVal")
                )],
            }
        },
        r###"<a test="testVal"/>"###
    )]
);

#[derive(Debug, PartialEq, SerializationGroup, DeserializationGroup)]
pub struct B {
    #[xgroup]
    any_attributes: AnyAttributes,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "a")]
pub struct C {
    #[xgroup]
    b: B,
}

define_test!(
    any_attributes_element_in_group,
    [(
        C {
            b: B {
                any_attributes: AnyAttributes {
                    attributes: vec![XmlAttribute::new(
                        ExpandedNameBuf::new(LocalNameBuf::from_str("test").unwrap(), None),
                        XmlText::new("testVal")
                    )],
                }
            }
        },
        r###"<a test="testVal"/>"###
    )]
);
