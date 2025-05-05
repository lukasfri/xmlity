use std::{io::Read, ops::Deref};

use xml::{
    name::OwnedName,
    reader::{EventReader, XmlEvent},
};

use xmlity::{
    de::{self, Error as _, Unexpected, Visitor},
    Deserialize, ExpandedName, LocalName, QName, XmlNamespace,
};

use crate::HasXmlRsAlternative;

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
        while let Ok(event) = self.reader.read_event() {
            match event {
                XmlEvent::EndDocument => return Ok(None),
                XmlEvent::Characters(text) if text.clone().into_inner().trim_ascii().is_empty() => {
                    continue;
                }
                event => return Ok(Some(event)),
            }
        }

        Ok(None)
    }

    fn read_until_element_end(&mut self, name: &QuickName, depth: i16) -> Result<(), Error> {
        while let Some(event) = self.peek_event() {
            let correct_name = match event {
                XmlEvent::EndElement { name: end_name } if end_name == *name => true,
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

    pub fn peek_event(&mut self) -> Option<&XmlEvent> {
        if self.peeked_event.is_some() {
            return self.peeked_event.as_ref();
        }

        self.peeked_event = self.read_event().ok().flatten();
        self.peeked_event.as_ref()
    }

    pub fn next_event(&mut self) -> Option<XmlEvent> {
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

    pub fn create_sub_seq_access<'p>(&'p mut self) -> SubSeqAccess<'p, R> {
        SubSeqAccess::Filled {
            current: Some(self.clone()),
            parent: self,
        }
    }

    pub fn try_deserialize<T, E>(
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

    pub fn expand_name<'a>(&self, qname: QuickName<'a>) -> ExpandedName<'a> {
        let (resolve_result, _) = self.reader.resolve(qname, false);
        let namespace = xml_namespace_from_resolve_result(resolve_result).map(|ns| ns.into_owned());

        ExpandedName::new(LocalName::from_quick_xml(qname.local_name()), namespace)
    }

    pub fn resolve_bytes_start<'a>(&self, bytes_start: &'a BytesStart<'a>) -> ExpandedName<'a> {
        self.expand_name(bytes_start.name())
    }

    pub fn resolve_attribute<'a>(&self, attribute: &'a Attribute<'a>) -> ExpandedName<'a> {
        self.expand_name(attribute.key)
    }
}

pub struct ElementAccess<'a, R: Read> {
    deserializer: Option<&'a mut Deserializer<R>>,
    attribute_index: usize,
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
            deserializer
                .read_until_element_end(&self.start_name.into_xmlity(), self.start_depth)?;
        }
        Ok(())
    }
}

pub struct AttributeAccess<'a> {
    name: ExpandedName<'a>,
    value: String,
}

impl<'a> de::AttributeAccess<'a> for AttributeAccess<'a> {
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

impl<'a> xmlity::Deserializer<'a> for AttributeDeserializer<'a> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'a>,
    {
        visitor.visit_attribute(AttributeAccess {
            name: self.name,
            value: self.value,
        })
    }

    fn deserialize_seq<V>(self, _: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'a>,
    {
        Err(Self::Error::Unexpected(de::Unexpected::Seq))
    }
}

pub struct SubAttributesAccess<'a, 'r, R: Read + 'r> {
    deserializer: &'a Deserializer<R>,
    bytes_start: &'a BytesStart<'r>,
    attribute_index: usize,
    write_attribute_to: &'a mut usize,
}

impl<R: Read> Drop for SubAttributesAccess<'_, '_, R> {
    fn drop(&mut self) {
        *self.write_attribute_to = self.attribute_index;
    }
}

fn next_attribute<'a, 'de, T: Deserialize<'de>, R: Read>(
    deserializer: &'a Deserializer<R>,
    bytes_start: &'a BytesStart<'_>,
    attribute_index: &'a mut usize,
) -> Result<Option<T>, Error> {
    let (attribute, key) = loop {
        let Some(attribute) = bytes_start.attributes().nth(*attribute_index) else {
            return Ok(None);
        };

        let attribute = attribute?;

        let key = deserializer.resolve_attribute(&attribute).into_owned();

        const XMLNS_NAMESPACE: XmlNamespace<'static> =
            XmlNamespace::new_dangerous("http://www.w3.org/2000/xmlns/");

        if key.namespace() == Some(&XMLNS_NAMESPACE) {
            *attribute_index += 1;
            continue;
        }

        break (attribute, key);
    };

    let value = String::from_utf8(attribute.value.into_owned())
        .expect("attribute value should be valid utf8");

    let deserializer = AttributeDeserializer { name: key, value };

    let res = T::deserialize(deserializer)?;

    // Only increment the index if the deserialization was successful
    *attribute_index += 1;

    Ok(Some(res))
}

impl<'de, R: Read> de::AttributesAccess<'de> for SubAttributesAccess<'_, 'de, R> {
    type Error = Error;

    type SubAccess<'a>
        = SubAttributesAccess<'a, 'de, R>
    where
        Self: 'a;

    fn next_attribute<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'de>,
    {
        next_attribute(
            self.deserializer,
            self.bytes_start,
            &mut self.attribute_index,
        )
    }

    fn sub_access(&mut self) -> Result<Self::SubAccess<'_>, Self::Error> {
        Ok(Self::SubAccess {
            deserializer: self.deserializer,
            bytes_start: self.bytes_start,
            attribute_index: self.attribute_index,
            write_attribute_to: self.write_attribute_to,
        })
    }
}

impl<'de> de::AttributesAccess<'de> for ElementAccess<'_, 'de> {
    type Error = Error;

    type SubAccess<'a>
        = SubAttributesAccess<'a, 'de>
    where
        Self: 'a;

    fn next_attribute<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'de>,
    {
        next_attribute(
            self.deserializer
                .as_ref()
                .expect("deserializer should be set"),
            &self.bytes_start,
            &mut self.attribute_index,
        )
    }

    fn sub_access(&mut self) -> Result<Self::SubAccess<'_>, Self::Error> {
        Ok(Self::SubAccess {
            bytes_start: &self.bytes_start,
            attribute_index: self.attribute_index,
            write_attribute_to: &mut self.attribute_index,
            deserializer: self
                .deserializer
                .as_ref()
                .expect("Should not be called after ElementAccess has been consumed"),
        })
    }
}

impl<'a, 'de, R: Read> de::ElementAccess<'de> for ElementAccess<'a, 'de, R> {
    type ChildrenAccess = ChildrenAccess<'a, R>;

    fn name(&self) -> ExpandedName<'_> {
        self.deserializer().resolve_bytes_start(&self.bytes_start)
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
                expected_end: QName::from_quick_xml(self.bytes_start.name()).into_owned(),
                start_depth: self.start_depth,
                deserializer,
            }
        })
    }
}

pub enum ChildrenAccess<'a, R: Read> {
    Filled {
        expected_end: QName<'static>,
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
            .read_until_element_end(&expected_end.into_xmlity(), *start_depth)
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
            if end_name.into_xmlity() != *expected_end && current_depth == *start_depth {
                return Err(Error::custom(format!(
                    "Expected end of element {}, found end of element {}",
                    expected_end,
                    end_name.into_xmlity()
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
            if *expected_end != end_name.into_xmlity() && current_depth == *start_depth {
                return Err(Error::custom(format!(
                    "Expected end of element {}, found end of element {}",
                    expected_end,
                    end_name.into_xmlity()
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
