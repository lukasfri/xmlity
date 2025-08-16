use std::{io::Read, ops::Deref};

use xml::{
    attribute::OwnedAttribute,
    name::OwnedName,
    reader::{EventReader, XmlEvent},
};

use xmlity::{
    de::{self, Error as _, Unexpected, Visitor},
    Deserialize, ExpandedName, QName, XmlNamespace,
};

use crate::{IsExpandedName, IsQName};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Quick XML error: {0}")]
    XmlRs(#[from] xml::reader::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Unexpected: {0}")]
    Unexpected(xmlity::de::Unexpected),
    #[error("Custom: {0}")]
    Custom(String),
    #[error("Wrong name: expected {expected:?}, got {actual:?}")]
    WrongName {
        actual: Box<ExpandedName<'static>>,
        expected: Box<ExpandedName<'static>>,
    },
    #[error("Unknown child")]
    UnknownChild,
    #[error("Invalid UTF-8: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
    #[error("Invalid string")]
    InvalidString,
    #[error("Missing field: {field}")]
    MissingField { field: String },
    #[error("No possible variant: {ident}")]
    NoPossibleVariant { ident: String },
    #[error("Missing data")]
    MissingData,
}

impl xmlity::de::Error for Error {
    fn custom<T: ToString>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }

    fn wrong_name(actual: &ExpandedName<'_>, expected: &ExpandedName<'_>) -> Self {
        Error::WrongName {
            actual: Box::new(actual.clone().into_owned()),
            expected: Box::new(expected.clone().into_owned()),
        }
    }

    fn unexpected_visit<T>(unexpected: xmlity::de::Unexpected, _expected: &T) -> Self {
        Error::Unexpected(unexpected)
    }

    fn missing_field(field: &str) -> Self {
        Error::MissingField {
            field: field.to_string(),
        }
    }

    fn no_possible_variant(ident: &str) -> Self {
        Error::NoPossibleVariant {
            ident: ident.to_string(),
        }
    }

    fn missing_data() -> Self {
        Error::MissingData
    }

    fn unknown_child() -> Self {
        Error::UnknownChild
    }

    fn invalid_string() -> Self {
        Error::InvalidString
    }
}

pub enum Peeked<'a> {
    None,
    Text,
    CData,
    Element {
        name: QName<'a>,
        namespace: Option<XmlNamespace<'a>>,
    },
}

pub struct Deserializer<R: Read> {
    reader: EventReader<R>,
    current_depth: i16,
    peeked_event: Option<XmlEvent>,
}

impl<R: Read> From<EventReader<R>> for Deserializer<R> {
    fn from(reader: EventReader<R>) -> Self {
        Self::new(reader)
    }
}

impl<R: Read> Deserializer<R> {
    pub fn new(reader: EventReader<R>) -> Self {
        Self {
            reader,
            current_depth: 0,
            peeked_event: None,
        }
    }

    fn read_event(&mut self) -> Result<Option<XmlEvent>, Error> {
        while let Ok(event) = self.reader.next() {
            match event {
                XmlEvent::EndDocument => return Ok(None),
                XmlEvent::Characters(text) if text.trim_ascii().is_empty() => {
                    continue;
                }
                event => return Ok(Some(event)),
            }
        }

        Ok(None)
    }

    fn read_until_element_end(&mut self, name: &OwnedName, depth: i16) -> Result<(), Error> {
        while let Some(event) = self.peek_event() {
            let correct_name = match event {
                XmlEvent::EndElement { name: end_name } if *end_name == *name => true,
                XmlEvent::EndDocument => return Err(Error::Unexpected(Unexpected::Eof)),
                _ => false,
            };

            if correct_name && self.current_depth == depth {
                return Ok(());
            }

            self.next_event();
        }

        Err(Error::Unexpected(de::Unexpected::Eof))
    }

    fn peek_event(&mut self) -> Option<&XmlEvent> {
        if self.peeked_event.is_some() {
            return self.peeked_event.as_ref();
        }

        self.peeked_event = self.read_event().ok().flatten();
        self.peeked_event.as_ref()
    }

    fn next_event(&mut self) -> Option<XmlEvent> {
        let event = if self.peeked_event.is_some() {
            self.peeked_event.take()
        } else {
            self.read_event().ok().flatten()
        };

        if matches!(event, Some(XmlEvent::EndElement { .. })) {
            self.current_depth -= 1;
        }
        if matches!(event, Some(XmlEvent::StartElement { .. })) {
            self.current_depth += 1;
        }

        event
    }

    fn create_sub_seq_access<'p>(&'p mut self) -> SubSeqAccess<'p, R> {
        SubSeqAccess::Filled {
            current: Some(self.clone()),
            parent: self,
        }
    }

    fn try_deserialize<T, E>(
        &mut self,
        closure: impl for<'a> FnOnce(&'a mut Deserializer<R>) -> Result<T, E>,
    ) -> Result<T, E> {
        let mut sub_deserializer = self.clone();
        let res = closure(&mut sub_deserializer);

        if res.is_ok() {
            *self = sub_deserializer;
        }
        res
    }
}

pub struct ElementAccess<'a, R: Read> {
    deserializer: Option<&'a mut Deserializer<R>>,
    attribute_index: usize,
    attributes: Vec<OwnedAttribute>,
    start_name: OwnedName,
    start_depth: i16,
    empty: bool,
}

impl<R: Read> Drop for ElementAccess<'_, R> {
    fn drop(&mut self) {
        self.try_end().ok();
    }
}

impl<R: Read> ElementAccess<'_, R> {
    fn deserializer(&self) -> &Deserializer<R> {
        self.deserializer
            .as_ref()
            .expect("Should not be called after ElementAccess has been consumed")
    }

    fn try_end(&mut self) -> Result<(), Error> {
        if self.empty {
            return Ok(());
        }

        if let Some(deserializer) = self.deserializer.as_mut() {
            deserializer.read_until_element_end(&self.start_name, self.start_depth)?;
        }
        Ok(())
    }
}

pub struct AttributeAccess<'a> {
    name: ExpandedName<'a>,
    value: String,
}

impl<'de> de::AttributeAccess<'de> for AttributeAccess<'_> {
    type Error = Error;

    fn name(&self) -> ExpandedName<'_> {
        self.name.clone()
    }

    fn value(&self) -> &str {
        self.value.as_str()
    }
}

struct EmptySeqAccess;

impl<'de> de::SeqAccess<'de> for EmptySeqAccess {
    type Error = Error;
    type SubAccess<'s>
        = EmptySeqAccess
    where
        Self: 's;

    fn next_element_seq<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'de>,
    {
        Ok(None)
    }

    fn next_element<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'de>,
    {
        Ok(None)
    }

    fn sub_access(&mut self) -> Result<Self::SubAccess<'_>, Self::Error> {
        Ok(EmptySeqAccess)
    }
}

struct AttributeDeserializer<'a> {
    name: ExpandedName<'a>,
    value: String,
}

impl<'de> xmlity::Deserializer<'de> for AttributeDeserializer<'_> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_attribute(AttributeAccess {
            name: self.name,
            value: self.value,
        })
    }

    fn deserialize_seq<V>(self, _: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Self::Error::Unexpected(de::Unexpected::Seq))
    }
}

pub struct SubAttributesAccess<'a, R: Read> {
    deserializer: &'a Deserializer<R>,
    attributes: &'a [OwnedAttribute],
    attribute_index: usize,
    write_attribute_to: &'a mut usize,
}

impl<R: Read> Drop for SubAttributesAccess<'_, R> {
    fn drop(&mut self) {
        *self.write_attribute_to = self.attribute_index;
    }
}

fn next_attribute<'a, 'de, T: Deserialize<'de>>(
    attributes: &[OwnedAttribute],
    attribute_index: &'a mut usize,
) -> Result<Option<T>, Error> {
    let (attribute, key) = loop {
        let Some(attribute) = attributes.get(*attribute_index) else {
            return Ok(None);
        };

        let key = (&attribute.name).into_expanded_name();

        const XMLNS_NAMESPACE: XmlNamespace<'static> =
            XmlNamespace::new_dangerous("http://www.w3.org/2000/xmlns/");

        if key.namespace() == Some(&XMLNS_NAMESPACE) {
            *attribute_index += 1;
            continue;
        }

        break (attribute, key);
    };

    let deserializer = AttributeDeserializer {
        name: key,
        value: attribute.value.clone(),
    };

    let res = T::deserialize(deserializer)?;

    // Only increment the index if the deserialization was successful
    *attribute_index += 1;

    Ok(Some(res))
}

impl<'de, R: Read + 'de> de::AttributesAccess<'de> for SubAttributesAccess<'_, R> {
    type Error = Error;

    type SubAccess<'a>
        = SubAttributesAccess<'a, R>
    where
        Self: 'a;

    fn next_attribute<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'de>,
    {
        next_attribute(
            // self.deserializer,
            self.attributes,
            &mut self.attribute_index,
        )
    }

    fn sub_access(&mut self) -> Result<Self::SubAccess<'_>, Self::Error> {
        Ok(Self::SubAccess {
            deserializer: self.deserializer,
            attributes: self.attributes,
            attribute_index: self.attribute_index,
            write_attribute_to: self.write_attribute_to,
        })
    }
}

impl<'de, R: Read + 'de> de::AttributesAccess<'de> for ElementAccess<'_, R> {
    type Error = Error;

    type SubAccess<'a>
        = SubAttributesAccess<'a, R>
    where
        Self: 'a;

    fn next_attribute<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'de>,
    {
        next_attribute(&self.attributes, &mut self.attribute_index)
    }

    fn sub_access(&mut self) -> Result<Self::SubAccess<'_>, Self::Error> {
        Ok(Self::SubAccess {
            attributes: &self.attributes,
            attribute_index: self.attribute_index,
            write_attribute_to: &mut self.attribute_index,
            deserializer: self
                .deserializer
                .as_ref()
                .expect("Should not be called after ElementAccess has been consumed"),
        })
    }
}

impl<'a, 'de, R: Read + 'de> de::ElementAccess<'de> for ElementAccess<'a, R> {
    type ChildrenAccess = ChildrenAccess<'a, R>;

    fn name(&self) -> ExpandedName<'_> {
        (&self.start_name).into_expanded_name()
    }

    fn children(mut self) -> Result<Self::ChildrenAccess, Self::Error> {
        Ok(if self.empty {
            ChildrenAccess::Empty
        } else {
            let deserializer = self
                .deserializer
                .take()
                .expect("Should not be called after ElementAccess has been consumed");

            ChildrenAccess::Filled {
                expected_end: self.start_name.clone(),
                start_depth: self.start_depth,
                deserializer,
            }
        })
    }
}

pub enum ChildrenAccess<'a, R: Read> {
    Filled {
        expected_end: OwnedName,
        deserializer: &'a mut Deserializer<R>,
        start_depth: i16,
    },
    Empty,
}

impl<R: Read> Drop for ChildrenAccess<'_, R> {
    fn drop(&mut self) {
        let ChildrenAccess::Filled {
            expected_end,
            deserializer,
            start_depth,
        } = self
        else {
            return;
        };

        deserializer
            .read_until_element_end(&expected_end, *start_depth)
            .unwrap();
    }
}

impl<'r, R: Read + 'r> de::SeqAccess<'r> for ChildrenAccess<'_, R> {
    type Error = Error;

    type SubAccess<'s>
        = SubSeqAccess<'s, R>
    where
        Self: 's;

    fn next_element<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'r>,
    {
        let ChildrenAccess::Filled {
            expected_end,
            deserializer,
            start_depth,
        } = self
        else {
            return Ok(None);
        };

        if deserializer.peek_event().is_none() {
            return Ok(None);
        }

        let current_depth = deserializer.current_depth;

        if let Some(XmlEvent::EndElement { name: end_name }) = deserializer.peek_event() {
            if end_name != expected_end && current_depth == *start_depth {
                return Err(Error::custom(format!(
                    "Expected end of element {}, found end of element {}",
                    expected_end,
                    end_name.into_qname()
                )));
            }

            return Ok(None);
        }

        deserializer
            .try_deserialize(|deserializer| Deserialize::<'r>::deserialize(deserializer))
            .map(Some)
    }

    fn next_element_seq<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'r>,
    {
        let ChildrenAccess::Filled {
            expected_end,
            deserializer,
            start_depth,
        } = self
        else {
            return Ok(None);
        };

        if deserializer.peek_event().is_none() {
            return Ok(None);
        }

        let current_depth = deserializer.current_depth;

        if let Some(XmlEvent::EndElement { name: end_name }) = deserializer.peek_event() {
            if *expected_end != *end_name && current_depth == *start_depth {
                return Err(Error::custom(format!(
                    "Expected end of element {}, found end of element {}",
                    expected_end,
                    end_name.into_qname()
                )));
            }

            return Ok(None);
        }

        deserializer
            .try_deserialize(|deserializer| Deserialize::<'r>::deserialize_seq(deserializer))
            .map(Some)
    }

    fn sub_access(&mut self) -> Result<Self::SubAccess<'_>, Self::Error> {
        let ChildrenAccess::Filled { deserializer, .. } = self else {
            return Ok(SubSeqAccess::Empty);
        };

        Ok(deserializer.create_sub_seq_access())
    }
}

pub struct SeqAccess<'a, R: Read> {
    deserializer: &'a mut Deserializer<R>,
}

#[allow(clippy::large_enum_variant)]
pub enum SubSeqAccess<'p, R: Read> {
    Filled {
        current: Option<Deserializer<R>>,
        parent: &'p mut Deserializer<R>,
    },
    Empty,
}

impl<R: Read> Drop for SubSeqAccess<'_, R> {
    fn drop(&mut self) {
        if let SubSeqAccess::Filled { current, parent } = self {
            **parent = current.take().expect("SubSeqAccess dropped twice");
        }
    }
}

impl<'r, R: Read + 'r> de::SeqAccess<'r> for SubSeqAccess<'_, R> {
    type Error = Error;

    type SubAccess<'s>
        = SubSeqAccess<'s, R>
    where
        Self: 's;

    fn next_element_seq<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'r>,
    {
        let Self::Filled { current, .. } = self else {
            return Ok(None);
        };

        let deserializer = current.as_mut().expect("SubSeqAccess used after drop");

        if deserializer.peek_event().is_none() {
            return Ok(None);
        }

        deserializer
            .try_deserialize(|deserializer| Deserialize::<'r>::deserialize_seq(deserializer))
            .map(Some)
    }

    fn next_element<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'r>,
    {
        let Self::Filled { current, .. } = self else {
            return Ok(None);
        };

        let deserializer = current.as_mut().expect("SubSeqAccess used after drop");

        if deserializer.peek_event().is_none() {
            return Ok(None);
        }

        deserializer
            .try_deserialize(|deserializer| Deserialize::<'r>::deserialize(deserializer))
            .map(Some)
    }

    fn sub_access(&mut self) -> Result<Self::SubAccess<'_>, Self::Error> {
        let Self::Filled { current, .. } = self else {
            return Ok(SubSeqAccess::Empty);
        };

        Ok(current
            .as_mut()
            .expect("SubSeqAccess used after drop")
            .create_sub_seq_access())
    }
}

impl<'r, R: Read + 'r> de::SeqAccess<'r> for SeqAccess<'_, R> {
    type Error = Error;

    type SubAccess<'s>
        = SubSeqAccess<'s, R>
    where
        Self: 's;

    fn next_element_seq<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'r>,
    {
        if self.deserializer.peek_event().is_none() {
            return Ok(None);
        }

        self.deserializer
            .try_deserialize(|deserializer| Deserialize::<'r>::deserialize_seq(deserializer))
            .map(Some)
    }

    fn next_element<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'r>,
    {
        if self.deserializer.peek_event().is_none() {
            return Ok(None);
        }

        self.deserializer
            .try_deserialize(|deserializer| Deserialize::<'r>::deserialize(deserializer))
            .map(Some)
    }

    fn sub_access(&mut self) -> Result<Self::SubAccess<'_>, Self::Error> {
        Ok(SubSeqAccess::Filled {
            current: Some(self.deserializer.clone()),
            parent: self.deserializer,
        })
    }
}

impl<'r, R: Read + 'r> xmlity::Deserializer<'r> for &mut Deserializer<R> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'r>,
    {
        let event = self.next_event().ok_or_else(|| Error::custom("EOF"))?;

        match event {
            XmlEvent::StartElement {
                name,
                namespace,
                attributes,
            } => {
                let value = Visitor::visit_element(
                    visitor,
                    ElementAccess {
                        start_name: name,
                        attributes,
                        start_depth: self.current_depth,
                        deserializer: Some(self),
                        empty: false,
                        attribute_index: 0,
                    },
                )?;

                let end_event = self.next_event().ok_or_else(|| Error::custom("EOF"))?;

                let success = if let XmlEvent::EndElement { name: end_name } = &end_event {
                    *end_name == name
                } else {
                    false
                };

                if success {
                    Ok(value)
                } else {
                    Err(Error::custom("No matching end element"))
                }
            }
            XmlEvent::EndElement { .. } => Err(Error::custom("Unexpected end element")),

            XmlEvent::Characters(bytes_text) => visitor.visit_text(bytes_text.deref()),
            XmlEvent::Whitespace(bytes_text) => visitor.visit_text(bytes_text.deref()),
            XmlEvent::CData(bytes_cdata) => visitor.visit_cdata(bytes_cdata.deref()),
            XmlEvent::Comment(bytes_text) => visitor.visit_comment(bytes_text.deref()),
            XmlEvent::StartDocument {
                encoding,
                standalone,
                version,
            } => visitor.visit_decl(
                version.to_string(),
                Some(encoding.to_string()),
                standalone.map(|standalone| standalone.to_string()),
            ),
            XmlEvent::ProcessingInstruction { data, name } => visitor.visit_pi(bytes_pi.deref()),
            XmlEvent::EndDocument => Err(Error::custom("Unexpected EOF")),
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'r>,
    {
        visitor.visit_seq(SeqAccess { deserializer: self })
    }
}

impl<'r, R: Read + 'r> xmlity::Deserializer<'r> for Deserializer<R> {
    type Error = Error;

    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'r>,
    {
        (&mut self).deserialize_any(visitor)
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'r>,
    {
        (&mut self).deserialize_seq(visitor)
    }
}
