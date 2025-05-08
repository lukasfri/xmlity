use std::{borrow::Cow, ops::Deref};

use quick_xml::{
    events::{
        attributes::Attribute, BytesCData, BytesDecl, BytesEnd, BytesPI, BytesStart, BytesText,
        Event,
    },
    name::QName as QuickName,
    NsReader,
};

use xmlity::{
    de::{
        self, Error as _, NamespaceContext, Unexpected, Visitor, XmlCData, XmlComment,
        XmlDeclaration, XmlDoctype, XmlProcessingInstruction, XmlText,
    },
    Deserialize, ExpandedName, LocalName, QName, XmlNamespace,
};

use crate::{xml_namespace_from_resolve_result, HasQuickXmlAlternative, OwnedQuickName};

/// Errors that can occur when using this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error from the `quick-xml` crate.
    #[error("Quick XML error: {0}")]
    QuickXml(#[from] quick_xml::Error),
    /// Error from the `quick-xml` crate when handling attributes.
    #[error("Attribute error: {0}")]
    AttrError(#[from] quick_xml::events::attributes::AttrError),
    /// IO errors.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// Unexpected segments that occurred when deserializing.
    #[error("Unexpected: {0}")]
    Unexpected(xmlity::de::Unexpected),
    /// Wrong name when trying to deserialize an element;
    #[error("Wrong name: expected {expected:?}, got {actual:?}")]
    WrongName {
        /// The actual name.
        actual: Box<ExpandedName<'static>>,
        /// The expected name.
        expected: Box<ExpandedName<'static>>,
    },
    /// Unknown child.
    #[error("Unknown child")]
    UnknownChild,
    /// Invalid string.
    #[error("Invalid string")]
    InvalidString,
    /// Missing field.
    #[error("Missing field: {field}")]
    MissingField {
        /// The name of the field.
        field: String,
    },
    /// No possible variant.
    #[error("No possible variant: {ident}")]
    NoPossibleVariant {
        /// The name of the enum.
        ident: String,
    },
    /// Missing data.
    #[error("Missing data")]
    MissingData,
    /// Custom errors occuring in [`Deserialize`] implementations.
    #[error("Custom: {0}")]
    Custom(String),
}

impl xmlity::de::Error for Error {
    fn custom<T: ToString>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }

    fn wrong_name(actual: &ExpandedName<'_>, expected: &ExpandedName<'_>) -> Self {
        Error::WrongName {
            actual: Box::new(actual.as_ref().into_owned()),
            expected: Box::new(expected.as_ref().into_owned()),
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

/// Deserialize from a string.
pub fn from_str<'a, T>(s: &'a str) -> Result<T, Error>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from(s.as_bytes());
    T::deserialize(&mut deserializer)
}

/// The [`xmlity::Deserializer`] for the `quick-xml` crate.
///
/// This currently only supports an underlying reader of type `&[u8]` due to limitations in the `quick-xml` crate.
#[derive(Debug, Clone)]
pub struct Deserializer<'i> {
    reader: NsReader<&'i [u8]>,
    current_depth: i16,
    peeked_event: Option<Event<'i>>,
}

impl<'i> From<NsReader<&'i [u8]>> for Deserializer<'i> {
    fn from(reader: NsReader<&'i [u8]>) -> Self {
        Self::new(reader)
    }
}

impl<'i> From<&'i [u8]> for Deserializer<'i> {
    fn from(buffer: &'i [u8]) -> Self {
        Self::new(NsReader::from_reader(buffer))
    }
}

impl<'i> Deserializer<'i> {
    /// Create a new deserializer from a [`NsReader<&'i [u8]>`].
    pub fn new(reader: NsReader<&'i [u8]>) -> Self {
        Self {
            reader,
            current_depth: 0,
            peeked_event: None,
        }
    }

    fn read_event(&mut self) -> Result<Option<Event<'i>>, Error> {
        match self.reader.read_event()? {
            Event::Eof => Ok(None),
            event => Ok(Some(event)),
        }
    }

    fn read_until_element_end(
        &mut self,
        name: quick_xml::name::QName,
        depth: i16,
    ) -> Result<(), Error> {
        while let Some(event) = self.peek_event() {
            let correct_name = match event {
                Event::End(ref e) if e.name() == name => true,
                Event::Eof => return Err(Error::Unexpected(Unexpected::Eof)),
                _ => false,
            };

            if correct_name && self.current_depth == depth {
                return Ok(());
            }

            self.next_event();
        }

        Err(Error::Unexpected(de::Unexpected::Eof))
    }

    fn peek_event(&mut self) -> Option<&Event<'i>> {
        if self.peeked_event.is_some() {
            return self.peeked_event.as_ref();
        }

        self.peeked_event = self.read_event().ok().flatten();
        self.peeked_event.as_ref()
    }

    fn next_event(&mut self) -> Option<Event<'i>> {
        let event = if self.peeked_event.is_some() {
            self.peeked_event.take()
        } else {
            self.read_event().ok().flatten()
        };

        if matches!(event, Some(Event::End(_))) {
            self.current_depth -= 1;
        }
        if matches!(event, Some(Event::Start(_))) {
            self.current_depth += 1;
        }

        event
    }

    fn create_sub_seq_access<'p>(&'p mut self) -> SubSeqAccess<'p, 'i> {
        SubSeqAccess::Filled {
            current: Some(self.clone()),
            parent: self,
        }
    }

    fn try_deserialize<T, E>(
        &mut self,
        closure: impl for<'a> FnOnce(&'a mut Deserializer<'i>) -> Result<T, E>,
    ) -> Result<T, E> {
        let mut sub_deserializer = self.clone();
        let res = closure(&mut sub_deserializer);

        if res.is_ok() {
            *self = sub_deserializer;
        }
        res
    }

    fn resolve_qname<'a>(&'a self, qname: QuickName<'a>) -> ExpandedName<'a> {
        let (resolve_result, _) = self.reader.resolve(qname, false);
        let namespace = xml_namespace_from_resolve_result(resolve_result);

        ExpandedName::new(LocalName::from_quick_xml(qname.local_name()), namespace)
    }

    fn resolve_bytes_start<'a>(&'a self, bytes_start: &'a BytesStart<'a>) -> ExpandedName<'a> {
        self.resolve_qname(bytes_start.name())
    }

    fn resolve_attribute<'a>(&'a self, attribute: &'a Attribute<'a>) -> ExpandedName<'a> {
        self.resolve_qname(attribute.key)
    }
}

struct ElementAccess<'a, 'de> {
    deserializer: Option<&'a mut Deserializer<'de>>,
    attribute_index: usize,
    bytes_start: Option<BytesStart<'de>>,
    start_depth: i16,
    empty: bool,
}

impl Drop for ElementAccess<'_, '_> {
    fn drop(&mut self) {
        self.try_end().ok();
    }
}

impl<'r> ElementAccess<'_, 'r> {
    fn deserializer(&self) -> &Deserializer<'r> {
        self.deserializer
            .as_ref()
            .expect("Should not be called after ElementAccess has been consumed")
    }

    fn try_end(&mut self) -> Result<(), Error> {
        if self.empty {
            return Ok(());
        }

        if let Some(deserializer) = self.deserializer.as_mut() {
            deserializer.read_until_element_end(
                self.bytes_start
                    .as_ref()
                    .expect("Should be some if deserializer is some")
                    .name(),
                self.start_depth,
            )?;
        }
        Ok(())
    }
}

impl NamespaceContext for &Deserializer<'_> {
    fn resolve_prefix(&self, prefix: xmlity::Prefix<'_>) -> Option<XmlNamespace<'_>> {
        let name = format!("{prefix}:a");
        let (_, namespace) = self.resolve_qname(QuickName(name.as_bytes())).into_parts();

        namespace.map(XmlNamespace::into_owned)
    }
}

struct AttributeAccess<'a> {
    name: ExpandedName<'a>,
    value: &'a str,
    deserializer: &'a Deserializer<'a>,
}

impl<'de> de::AttributeAccess<'de> for AttributeAccess<'_> {
    type Error = Error;

    type NamespaceContext<'b>
        = &'b Deserializer<'b>
    where
        Self: 'b;

    fn name(&self) -> ExpandedName<'_> {
        self.name.as_ref()
    }

    fn value(&self) -> &str {
        self.value
    }

    fn namespace_context<'a>(&'a self) -> Self::NamespaceContext<'a> {
        self.deserializer
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
    value: &'a str,
    deserializer: &'a Deserializer<'a>,
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
            deserializer: self.deserializer,
        })
    }

    fn deserialize_seq<V>(self, _: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Self::Error::Unexpected(de::Unexpected::Seq))
    }
}

struct SubAttributesAccess<'a, 'r> {
    deserializer: &'a Deserializer<'r>,
    bytes_start: &'a BytesStart<'r>,
    attribute_index: usize,
    write_attribute_to: &'a mut usize,
}

impl Drop for SubAttributesAccess<'_, '_> {
    fn drop(&mut self) {
        *self.write_attribute_to = self.attribute_index;
    }
}

fn next_attribute<'a, 'de, T: Deserialize<'de>>(
    deserializer: &'a Deserializer<'_>,
    bytes_start: &'a BytesStart<'_>,
    attribute_index: &'a mut usize,
) -> Result<Option<T>, Error> {
    while let Some(attribute) = bytes_start.attributes().nth(*attribute_index) {
        let attribute = attribute?;

        let key = deserializer.resolve_attribute(&attribute);

        if key.namespace() == Some(&XmlNamespace::XMLNS) {
            *attribute_index += 1;
            continue;
        }

        let value =
            std::str::from_utf8(&attribute.value).expect("attribute value should be valid utf8");

        let deserializer = AttributeDeserializer {
            name: key,
            value,
            deserializer,
        };

        let res = T::deserialize(deserializer)?;

        // Only increment the index if the deserialization was successful
        *attribute_index += 1;

        return Ok(Some(res));
    }

    Ok(None)
}

impl<'de> de::AttributesAccess<'de> for SubAttributesAccess<'_, 'de> {
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
            self.bytes_start
                .as_ref()
                .expect("bytes_start should be set"),
            &mut self.attribute_index,
        )
    }

    fn sub_access(&mut self) -> Result<Self::SubAccess<'_>, Self::Error> {
        Ok(Self::SubAccess {
            bytes_start: self
                .bytes_start
                .as_ref()
                .expect("Should not be called after ElementAccess has been consumed"),
            attribute_index: self.attribute_index,
            write_attribute_to: &mut self.attribute_index,
            deserializer: self
                .deserializer
                .as_ref()
                .expect("Should not be called after ElementAccess has been consumed"),
        })
    }
}

impl<'a, 'de> de::ElementAccess<'de> for ElementAccess<'a, 'de> {
    type ChildrenAccess = ChildrenAccess<'a, 'de>;
    type NamespaceContext<'b>
        = &'b Deserializer<'de>
    where
        Self: 'b;

    fn name(&self) -> ExpandedName<'_> {
        self.deserializer().resolve_bytes_start(
            self.bytes_start
                .as_ref()
                .expect("bytes_start should be set"),
        )
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
                expected_end: self
                    .bytes_start
                    .take()
                    .expect("Should not be called after ElementAccess has been consumed"),
                start_depth: self.start_depth,
                deserializer,
            }
        })
    }

    fn namespace_context(&self) -> Self::NamespaceContext<'_> {
        self.deserializer()
    }
}

enum ChildrenAccess<'a, 'de> {
    Filled {
        expected_end: BytesStart<'de>,
        deserializer: &'a mut Deserializer<'de>,
        start_depth: i16,
    },
    Empty,
}

impl Drop for ChildrenAccess<'_, '_> {
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
            .read_until_element_end(expected_end.name(), *start_depth)
            .unwrap();
    }
}

impl ChildrenAccess<'_, '_> {
    fn check_end<T>(
        expected_end: &BytesStart,
        bytes_end: &BytesEnd,
        current_depth: i16,
        start_depth: i16,
    ) -> Result<Option<T>, Error> {
        if expected_end.name() != bytes_end.name() && current_depth == start_depth {
            return Err(Error::custom(format!(
                "Expected end of element {}, found end of element {}",
                QName::from_quick_xml(expected_end.name()),
                QName::from_quick_xml(bytes_end.name())
            )));
        }
        Ok(None)
    }
}

impl<'r> de::SeqAccess<'r> for ChildrenAccess<'_, 'r> {
    type Error = Error;

    type SubAccess<'s>
        = SubSeqAccess<'s, 'r>
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

        if let Some(Event::End(bytes_end)) = deserializer.peek_event() {
            return Self::check_end(expected_end, bytes_end, current_depth, *start_depth);
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

        if let Some(Event::End(bytes_end)) = deserializer.peek_event() {
            return Self::check_end(expected_end, bytes_end, current_depth, *start_depth);
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

struct SeqAccess<'a, 'r> {
    deserializer: &'a mut Deserializer<'r>,
}

#[allow(clippy::large_enum_variant)]
enum SubSeqAccess<'p, 'r> {
    Filled {
        current: Option<Deserializer<'r>>,
        parent: &'p mut Deserializer<'r>,
    },
    Empty,
}

impl Drop for SubSeqAccess<'_, '_> {
    fn drop(&mut self) {
        if let SubSeqAccess::Filled { current, parent } = self {
            **parent = current.take().expect("SubSeqAccess dropped twice");
        }
    }
}

impl<'r> de::SeqAccess<'r> for SubSeqAccess<'_, 'r> {
    type Error = Error;

    type SubAccess<'s>
        = SubSeqAccess<'s, 'r>
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

impl<'r> de::SeqAccess<'r> for SeqAccess<'_, 'r> {
    type Error = Error;

    type SubAccess<'s>
        = SubSeqAccess<'s, 'r>
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

struct DataWithD<'a, T> {
    data: T,
    deserializer: &'a Deserializer<'a>,
}

impl<'a, T> DataWithD<'a, T> {
    fn new(data: T, deserializer: &'a Deserializer<'a>) -> Self {
        Self { data, deserializer }
    }
}

impl<'de> XmlText<'de> for DataWithD<'_, BytesText<'de>> {
    type NamespaceContext<'a>
        = &'a Deserializer<'a>
    where
        Self: 'a;

    fn into_bytes(self) -> Cow<'de, [u8]> {
        self.data.into_inner()
    }

    fn as_bytes(&self) -> &[u8] {
        self.data.deref()
    }

    fn into_string(self) -> Cow<'de, str> {
        match self.data.into_inner() {
            Cow::Borrowed(bytes) => Cow::Borrowed(std::str::from_utf8(bytes).unwrap()),
            Cow::Owned(bytes) => Cow::Owned(std::string::String::from_utf8(bytes).unwrap()),
        }
    }

    fn as_str(&self) -> &str {
        std::str::from_utf8(self.data.deref()).unwrap()
    }

    fn namespace_context(&self) -> Self::NamespaceContext<'_> {
        self.deserializer
    }
}

impl XmlCData for DataWithD<'_, BytesCData<'_>> {
    type NamespaceContext<'a>
        = &'a Deserializer<'a>
    where
        Self: 'a;

    fn as_bytes(&self) -> &[u8] {
        self.data.deref()
    }

    fn as_str(&self) -> Cow<'_, str> {
        Cow::Borrowed(std::str::from_utf8(self.data.deref()).unwrap())
    }

    fn namespace_context(&self) -> Self::NamespaceContext<'_> {
        self.deserializer
    }
}

impl XmlComment for DataWithD<'_, BytesText<'_>> {
    type NamespaceContext<'a>
        = &'a Deserializer<'a>
    where
        Self: 'a;

    fn as_bytes(&self) -> &[u8] {
        self.data.deref()
    }

    fn namespace_context(&self) -> Self::NamespaceContext<'_> {
        self.deserializer
    }
}

struct ClearedByteDecl<'a> {
    version: Cow<'a, [u8]>,
    encoding: Option<Cow<'a, [u8]>>,
    standalone: Option<Cow<'a, [u8]>>,
}

impl<'a> TryFrom<&'a BytesDecl<'a>> for ClearedByteDecl<'a> {
    type Error = Error;

    fn try_from(bytes_decl: &'a BytesDecl<'a>) -> Result<Self, Self::Error> {
        Ok(Self {
            version: bytes_decl.version()?,
            encoding: match bytes_decl.encoding() {
                Some(Ok(encoding)) => Some(encoding),
                Some(Err(err)) => return Err(Error::QuickXml(err.into())),
                None => None,
            },
            standalone: match bytes_decl.standalone() {
                Some(Ok(standalone)) => Some(standalone),
                Some(Err(err)) => return Err(Error::QuickXml(err.into())),
                None => None,
            },
        })
    }
}

impl XmlDeclaration for DataWithD<'_, ClearedByteDecl<'_>> {
    type NamespaceContext<'a>
        = &'a Deserializer<'a>
    where
        Self: 'a;

    fn version(&self) -> &[u8] {
        self.data.version.as_ref()
    }

    fn encoding(&self) -> Option<&[u8]> {
        self.data.encoding.as_deref()
    }

    fn standalone(&self) -> Option<&[u8]> {
        self.data.standalone.as_deref()
    }

    fn namespace_context(&self) -> Self::NamespaceContext<'_> {
        self.deserializer
    }
}

impl XmlProcessingInstruction for DataWithD<'_, BytesPI<'_>> {
    type NamespaceContext<'a>
        = &'a Deserializer<'a>
    where
        Self: 'a;

    fn target(&self) -> &[u8] {
        self.data.target()
    }

    fn content(&self) -> &[u8] {
        self.data.content()
    }

    fn namespace_context(&self) -> Self::NamespaceContext<'_> {
        self.deserializer
    }
}

impl XmlDoctype for DataWithD<'_, BytesText<'_>> {
    type NamespaceContext<'a>
        = &'a Deserializer<'a>
    where
        Self: 'a;

    fn as_bytes(&self) -> &[u8] {
        self.data.deref()
    }

    fn namespace_context(&self) -> Self::NamespaceContext<'_> {
        self.deserializer
    }
}

impl<'r> xmlity::Deserializer<'r> for &mut Deserializer<'r> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'r>,
    {
        let event = self.next_event().ok_or_else(|| Error::custom("EOF"))?;

        match event {
            Event::Start(bytes_start) => {
                let element_name = OwnedQuickName(bytes_start.name().0.to_owned());

                let value = Visitor::visit_element(
                    visitor,
                    ElementAccess {
                        bytes_start: Some(bytes_start),
                        start_depth: self.current_depth,
                        deserializer: Some(self),
                        empty: false,
                        attribute_index: 0,
                    },
                )?;

                let end_event = self.next_event().ok_or_else(|| Error::custom("EOF"))?;

                let success = if let Event::End(bytes_end) = &end_event {
                    bytes_end.name() == element_name.as_ref()
                } else {
                    false
                };

                if success {
                    Ok(value)
                } else {
                    Err(Error::custom("No matching end element"))
                }
            }
            Event::End(_bytes_end) => Err(Error::custom("Unexpected end element")),
            Event::Empty(bytes_start) => visitor.visit_element(ElementAccess {
                bytes_start: Some(bytes_start),
                start_depth: self.current_depth,
                deserializer: Some(self),
                empty: true,
                attribute_index: 0,
            }),
            Event::Text(bytes_text) => visitor.visit_text(DataWithD::new(bytes_text, self)),
            Event::CData(bytes_cdata) => visitor.visit_cdata(DataWithD::new(bytes_cdata, self)),
            Event::Comment(bytes_text) => visitor.visit_comment(DataWithD::new(bytes_text, self)),
            Event::Decl(bytes_decl) => visitor.visit_decl(DataWithD::new(
                ClearedByteDecl::try_from(&bytes_decl)?,
                self,
            )),
            Event::PI(bytes_pi) => visitor.visit_pi(DataWithD::new(bytes_pi, self)),
            Event::DocType(bytes_text) => visitor.visit_doctype(DataWithD::new(bytes_text, self)),
            Event::Eof => Err(Error::custom("Unexpected EOF")),
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'r>,
    {
        visitor.visit_seq(SeqAccess { deserializer: self })
    }
}

impl<'r> xmlity::Deserializer<'r> for Deserializer<'r> {
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
