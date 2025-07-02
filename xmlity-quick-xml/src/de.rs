/// The [`xmlity::de::Deserializer`] implementation for the `quick-xml` crate.
///
/// This deserializer is based upon the [`quick_xml::NsReader`] with the same limits as the underlying reader, including requiring a `[u8]` backing.
use std::{borrow::Cow, ops::Deref};

use quick_xml::{
    events::{attributes::Attribute, BytesCData, BytesDecl, BytesPI, BytesStart, BytesText, Event},
    name::QName as QuickName,
    NsReader,
};

use xmlity::{
    de::{
        self, Error as _, NamespaceContext, Visitor, XmlCData, XmlComment, XmlDeclaration,
        XmlDoctype, XmlProcessingInstruction, XmlText,
    },
    Deserialize, ExpandedName, LocalName, XmlNamespace,
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
    /// Start element without end.
    #[error("Start element without end: {name}")]
    StartElementWithoutEnd {
        /// The name of the start element.
        name: String,
    },
    /// No matching end element.
    #[error("No matching end element")]
    NoMatchingEndElement {
        /// The name of the start element.
        start_name: String,
        /// The name of the end element.
        end_name: String,
    },
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

/// This reader wraps the `quick_xml::NsReader` and provides a way to peek and read events from the XML stream, as well as observe the depth, which are properties used when deserializing.
#[derive(Debug, Clone)]
struct Reader<'i> {
    reader: NsReader<&'i [u8]>,
    current_depth: i16,
    peeked_event: Option<Event<'i>>,
}
impl<'i> Reader<'i> {
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

    pub fn peek_event(&mut self) -> Result<Option<&Event<'i>>, Error> {
        if self.peeked_event.is_some() {
            return Ok(self.peeked_event.as_ref());
        }

        self.peeked_event = self.read_event()?;
        Ok(self.peeked_event.as_ref())
    }

    pub fn next_event(&mut self) -> Result<Option<Event<'i>>, Error> {
        let event = if self.peeked_event.is_some() {
            self.peeked_event.take()
        } else {
            self.read_event()?
        };

        if matches!(event, Some(Event::End(_))) {
            self.current_depth -= 1;
        }
        if matches!(event, Some(Event::Start(_))) {
            self.current_depth += 1;
        }

        Ok(event)
    }

    pub fn resolve_qname<'a>(&'a self, qname: QuickName<'a>, attribute: bool) -> ExpandedName<'a> {
        let (resolve_result, _) = self.reader.resolve(qname, attribute);
        let namespace = xml_namespace_from_resolve_result(resolve_result);

        ExpandedName::new(LocalName::from_quick_xml(qname.local_name()), namespace)
    }

    pub fn resolve_bytes_start<'a>(&'a self, bytes_start: &'a BytesStart<'a>) -> ExpandedName<'a> {
        self.resolve_qname(bytes_start.name(), false)
    }

    pub fn resolve_attribute<'a>(&'a self, attribute: &'a Attribute<'a>) -> ExpandedName<'a> {
        self.resolve_qname(attribute.key, true)
    }

    pub fn current_depth(&self) -> i16 {
        self.current_depth
    }
}

/// The [`xmlity::Deserializer`] for the `quick-xml` crate.
///
/// This currently only supports an underlying reader of type `&[u8]` due to limitations in the `quick-xml` crate.
#[derive(Debug, Clone)]
pub struct Deserializer<'i> {
    reader: Reader<'i>,
    // Limit depth
    limit_depth: i16,
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
            reader: Reader::new(reader),
            limit_depth: 0,
        }
    }

    fn read_until_end(&mut self) -> Result<(), Error> {
        while let Some(event) = self.next_event() {
            debug_assert!(!matches!(event, Event::Eof));
        }

        Ok(())
    }

    fn peek_event(&mut self) -> Option<&Event<'i>> {
        if self.reader.current_depth() < self.limit_depth {
            return None;
        }

        self.reader.peek_event().ok().flatten()
    }

    fn next_event(&mut self) -> Option<Event<'i>> {
        // Peek to check if we've reached the limit depth/limit end
        if self.reader.current_depth() < self.limit_depth {
            return None;
        }

        if self.reader.current_depth() == self.limit_depth
            && matches!(self.peek_event(), Some(Event::End(_)))
        {
            return None;
        }

        self.reader.next_event().ok().flatten()
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

    fn sub_deserializer(&mut self, limit_depth: i16) -> Self {
        Self {
            reader: self.reader.clone(),
            limit_depth,
        }
    }

    fn resolve_qname<'a>(&'a self, qname: QuickName<'a>, attribute: bool) -> ExpandedName<'a> {
        self.reader.resolve_qname(qname, attribute)
    }

    fn resolve_bytes_start<'a>(&'a self, bytes_start: &'a BytesStart<'a>) -> ExpandedName<'a> {
        self.reader.resolve_bytes_start(bytes_start)
    }

    fn resolve_attribute<'a>(&'a self, attribute: &'a Attribute<'a>) -> ExpandedName<'a> {
        self.reader.resolve_attribute(attribute)
    }
}

struct ElementAccess<'a, 'de> {
    deserializer: Option<&'a mut Deserializer<'de>>,
    attribute_index: usize,
    bytes_start: Option<BytesStart<'de>>,
    start_depth: i16,
    empty: bool,
}

impl<'r> ElementAccess<'_, 'r> {
    fn deserializer(&self) -> &Deserializer<'r> {
        self.deserializer
            .as_ref()
            .expect("Should not be called after ElementAccess has been consumed")
    }
}

const PLACEHOLDER_ELEMENT_NAME: &str = "a";

impl NamespaceContext for &Deserializer<'_> {
    fn default_namespace(&self) -> Option<XmlNamespace<'_>> {
        let (_, namespace) = self
            .resolve_qname(QuickName(PLACEHOLDER_ELEMENT_NAME.as_bytes()), false)
            .into_parts();

        namespace.map(XmlNamespace::into_owned)
    }

    fn resolve_prefix(&self, prefix: xmlity::Prefix<'_>) -> Option<XmlNamespace<'_>> {
        let name = format!("{prefix}:{PLACEHOLDER_ELEMENT_NAME}");
        let (_, namespace) = self
            .resolve_qname(QuickName(name.as_bytes()), false)
            .into_parts();

        namespace.map(XmlNamespace::into_owned)
    }
}

struct AttributeAccess<'a, 'v> {
    name: ExpandedName<'v>,
    value: Cow<'v, [u8]>,
    deserializer: &'a Deserializer<'a>,
}

impl<'de> de::AttributeAccess<'de> for AttributeAccess<'_, 'de> {
    type Error = Error;

    fn name(&self) -> ExpandedName<'de> {
        self.name.clone()
    }

    /// Deserializes the value of the attribute.
    fn value<T>(self) -> Result<T, Self::Error>
    where
        T: Deserialize<'de>,
    {
        T::deserialize(TextDeserializer {
            value: self.value,
            deserializer: self.deserializer,
            used_up: false,
        })
    }
}

struct TextDeserializer<'a, 'v> {
    value: Cow<'v, [u8]>,
    deserializer: &'a Deserializer<'a>,
    used_up: bool,
}

impl<'de> de::XmlText<'de> for TextDeserializer<'_, 'de> {
    type NamespaceContext<'a>
        = &'a Deserializer<'a>
    where
        Self: 'a;

    fn into_bytes(self) -> Cow<'de, [u8]> {
        self.value
    }

    fn as_bytes(&self) -> &[u8] {
        self.value.as_ref()
    }

    fn into_string(self) -> Cow<'de, str> {
        match self.value {
            Cow::Borrowed(bytes) => Cow::Borrowed(std::str::from_utf8(bytes).unwrap()),
            Cow::Owned(_) => Cow::Owned(String::from_utf8(self.value.into_owned()).unwrap()),
        }
    }

    fn as_str(&self) -> &str {
        std::str::from_utf8(self.value.as_ref()).unwrap()
    }

    fn namespace_context(&self) -> Self::NamespaceContext<'_> {
        self.deserializer
    }
}

impl<'de> de::SeqAccess<'de> for TextDeserializer<'_, 'de> {
    type Error = Error;

    type SubAccess<'g>
        = TextDeserializer<'g, 'de>
    where
        Self: 'g;

    fn next_element<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'de>,
    {
        if self.used_up {
            return Ok(None);
        }

        T::deserialize(TextDeserializer {
            value: self.value.clone(),
            deserializer: self.deserializer,
            used_up: false,
        })
        .map(|value| {
            self.used_up = true;
            Some(value)
        })
    }

    fn next_element_seq<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'de>,
    {
        if self.used_up {
            return Ok(None);
        }

        T::deserialize_seq(TextDeserializer {
            value: self.value.clone(),
            deserializer: self.deserializer,
            used_up: false,
        })
        .map(|value| {
            self.used_up = true;
            Some(value)
        })
    }

    fn sub_access(&mut self) -> Result<Self::SubAccess<'_>, Self::Error> {
        Ok(TextDeserializer {
            value: self.value.clone(),
            deserializer: self.deserializer,
            used_up: self.used_up,
        })
    }
}

impl<'de> de::Deserializer<'de> for TextDeserializer<'_, 'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_text(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(self)
    }
}

struct AttributeDeserializer<'a, 'v> {
    name: ExpandedName<'v>,
    value: Cow<'v, [u8]>,
    deserializer: &'a Deserializer<'a>,
}

impl<'de> xmlity::Deserializer<'de> for AttributeDeserializer<'_, 'de> {
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

fn key_is_declaration(key: &ExpandedName) -> bool {
    key.namespace() == Some(&XmlNamespace::XMLNS)
        || (key.local_name() == &LocalName::new_dangerous("xmlns") && key.namespace().is_none())
}

fn next_attribute<'a, 'de, T: Deserialize<'de>>(
    deserializer: &'a Deserializer<'_>,
    bytes_start: &'a BytesStart<'_>,
    attribute_index: &'a mut usize,
) -> Result<Option<T>, Error> {
    while let Some(attribute) = bytes_start.attributes().nth(*attribute_index) {
        let attribute = attribute?;

        let key = deserializer.resolve_attribute(&attribute).into_owned();

        if key_is_declaration(&key) {
            *attribute_index += 1;
            continue;
        }

        let deserializer: AttributeDeserializer<'_, 'static> = AttributeDeserializer {
            name: key,
            value: Cow::Owned(attribute.value.into_owned()),
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
    type ChildrenAccess = SeqAccess<'a, 'de>;
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
            SeqAccess::Empty
        } else {
            let deserializer = self
                .deserializer
                .take()
                .expect("Should not be called after ElementAccess has been consumed");

            SeqAccess::Filled {
                current: Some(deserializer.sub_deserializer(self.start_depth)),
                parent: deserializer,
            }
        })
    }

    fn namespace_context(&self) -> Self::NamespaceContext<'_> {
        self.deserializer()
    }
}

#[allow(clippy::large_enum_variant)]
enum SeqAccess<'p, 'r> {
    Filled {
        current: Option<Deserializer<'r>>,
        parent: &'p mut Deserializer<'r>,
    },
    Empty,
}

impl<'gp, 'i> SeqAccess<'gp, 'i> {
    fn create_sub_seq_access<'p>(&'p mut self) -> SeqAccess<'p, 'i> {
        match self {
            SeqAccess::Filled { current, .. } => {
                let current = current.as_mut().expect("SubSeqAccess used after drop");
                SeqAccess::Filled {
                    current: Some(current.clone()),
                    parent: current,
                }
            }
            SeqAccess::Empty => SeqAccess::Empty,
        }
    }
}

impl Drop for SeqAccess<'_, '_> {
    fn drop(&mut self) {
        if let SeqAccess::Filled {
            current, parent, ..
        } = self
        {
            parent.reader = current.take().expect("SubSeqAccess dropped twice").reader;
        }
    }
}

impl<'r> de::SeqAccess<'r> for SeqAccess<'_, 'r> {
    type Error = Error;

    type SubAccess<'s>
        = SeqAccess<'s, 'r>
    where
        Self: 's;

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

        if let Some(Event::End(_)) = deserializer.peek_event() {
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
        let Self::Filled { current, .. } = self else {
            return Ok(None);
        };

        let deserializer = current.as_mut().expect("SubSeqAccess used after drop");

        if deserializer.peek_event().is_none() {
            return Ok(None);
        }

        if let Some(Event::End(_)) = deserializer.peek_event() {
            return Ok(None);
        }

        deserializer
            .try_deserialize(|deserializer| Deserialize::<'r>::deserialize_seq(deserializer))
            .map(Some)
    }

    fn sub_access(&mut self) -> Result<Self::SubAccess<'_>, Self::Error> {
        Ok(self.create_sub_seq_access())
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

impl<'de> XmlCData<'de> for DataWithD<'_, BytesCData<'de>> {
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

impl<'de> XmlComment<'de> for DataWithD<'_, BytesText<'de>> {
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

impl<'de> XmlDoctype<'de> for DataWithD<'_, BytesText<'de>> {
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
        let Some(event) = self.next_event() else {
            return visitor.visit_none();
        };

        match event {
            Event::Start(bytes_start) => {
                let element_name = OwnedQuickName(bytes_start.name().0.to_owned());

                let mut sub = self.sub_deserializer(self.reader.current_depth());

                let element = ElementAccess {
                    bytes_start: Some(bytes_start),
                    start_depth: self.reader.current_depth(),
                    deserializer: Some(&mut sub),
                    empty: false,
                    attribute_index: 0,
                };

                let value = visitor.visit_element(element)?;

                sub.read_until_end()?;

                self.reader = sub.reader;

                let end_event = self
                    .next_event()
                    .ok_or_else(|| Error::StartElementWithoutEnd {
                        name: String::from_utf8_lossy(element_name.0.as_slice()).to_string(),
                    })?;

                if let Event::End(bytes_end) = &end_event {
                    if bytes_end.name() == element_name.as_ref() {
                        Ok(value)
                    } else {
                        Err(Error::NoMatchingEndElement {
                            start_name: String::from_utf8_lossy(element_name.0.as_slice())
                                .to_string(),
                            end_name: String::from_utf8_lossy(bytes_end.name().0).to_string(),
                        })
                    }
                } else {
                    Err(Error::StartElementWithoutEnd {
                        name: String::from_utf8_lossy(element_name.0.as_slice()).to_string(),
                    })
                }
            }
            Event::End(_bytes_end) => Err(Error::custom("Unexpected end element")),
            Event::Empty(bytes_start) => visitor.visit_element(ElementAccess {
                bytes_start: Some(bytes_start),
                start_depth: self.reader.current_depth(),
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
        if self.peek_event().is_some() {
            visitor.visit_seq(SeqAccess::Filled {
                current: Some(self.clone()),
                parent: self,
            })
        } else {
            visitor.visit_none()
        }
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
