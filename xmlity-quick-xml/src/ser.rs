use core::str;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::io::Write;
use std::ops::DerefMut;

use quick_xml::events::{BytesCData, BytesDecl, BytesEnd, BytesPI, BytesStart, BytesText, Event};
use quick_xml::writer::Writer as QuickXmlWriter;

use xmlity::ser::IncludePrefix;
use xmlity::{ser, ExpandedName, LocalName, Prefix, QName, Serialize, XmlNamespace};

use crate::{OwnedQuickName, XmlnsDeclaration};

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
    /// Custom errors from [`Serialize`] implementations.
    #[error("Custom: {0}")]
    Custom(String),
    /// Invalid UTF-8 when serializing.
    #[error("Invalid UTF-8: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
}

impl xmlity::ser::Error for Error {
    fn custom<T: ToString>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}

fn serializer_to_string<T>(serializer: QuickXmlWriter<Vec<u8>>, value: &T) -> Result<String, Error>
where
    T: Serialize,
{
    let mut serializer = Serializer::from(serializer);
    value.serialize(&mut serializer)?;
    let bytes = serializer.into_inner();

    String::from_utf8(bytes).map_err(Error::InvalidUtf8)
}

/// Serialize a value into a string.
pub fn to_string<T>(value: &T) -> Result<String, Error>
where
    T: Serialize,
{
    serializer_to_string(QuickXmlWriter::new(Vec::new()), value)
}

/// Serialize a value into a string with pretty printing.
pub fn to_string_pretty<T>(value: &T, indentation: usize) -> Result<String, Error>
where
    T: Serialize,
{
    serializer_to_string(
        QuickXmlWriter::new_with_indent(Vec::new(), b' ', indentation),
        value,
    )
}

struct NamespaceScope<'a> {
    pub defined_namespaces: BTreeMap<Prefix<'a>, XmlNamespace<'a>>,
}

impl<'a> NamespaceScope<'a> {
    pub fn new() -> Self {
        Self {
            defined_namespaces: BTreeMap::new(),
        }
    }

    const XML_PREFIX: Prefix<'static> = Prefix::new_dangerous("xml");
    const XML_NAMESPACE: XmlNamespace<'static> =
        XmlNamespace::new_dangerous("http://www.w3.org/XML/1998/namespace");

    pub fn top_scope() -> Self {
        let mut scope = Self::new();
        scope
            .defined_namespaces
            .insert(Self::XML_PREFIX, Self::XML_NAMESPACE);
        scope
    }

    pub fn get_namespace<'b>(&'b self, prefix: &'b Prefix<'b>) -> Option<&'b XmlNamespace<'a>> {
        self.defined_namespaces.get(prefix)
    }
}

struct NamespaceScopeContainer<'a> {
    scopes: Vec<NamespaceScope<'a>>,
    prefix_generator: PrefixGenerator,
}

struct PrefixGenerator {
    count: usize,
}

impl PrefixGenerator {
    pub fn index_to_name(index: usize) -> Prefix<'static> {
        // 0 = a0
        // 1 = a1
        // 26 = b0
        // 27 = b1
        // 52 = c0
        // 53 = c1
        // ...

        let letter = (index / 26) as u8 + b'a';
        let number = (index % 26) as u8 + b'0';
        let mut name = String::with_capacity(2);
        name.push(letter as char);
        name.push(number as char);
        Prefix::new(name).expect("Invalid prefix generated")
    }

    pub fn new() -> Self {
        Self { count: 0 }
    }

    pub fn new_prefix(&mut self) -> Prefix<'static> {
        let name = Self::index_to_name(self.count);
        self.count += 1;
        name
    }
}

impl<'a> NamespaceScopeContainer<'a> {
    pub fn new() -> Self {
        Self {
            scopes: vec![NamespaceScope::top_scope()],
            prefix_generator: PrefixGenerator::new(),
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(NamespaceScope::new())
    }

    pub fn pop_scope(&mut self) -> Option<NamespaceScope> {
        self.scopes.pop()
    }

    pub fn get_namespace<'b>(&'b self, prefix: &'b Prefix<'b>) -> Option<&'b XmlNamespace<'a>> {
        self.scopes
            .iter()
            .rev()
            .find_map(|a| a.get_namespace(prefix))
    }

    /// Find matching prefix
    pub fn find_matching_namespace<'b>(
        &'b self,
        namespace: &'_ XmlNamespace<'_>,
    ) -> Option<&'b Prefix<'a>> {
        self.scopes.iter().rev().find_map(|a| {
            a.defined_namespaces
                .iter()
                .find(|(_, found_namespace)| namespace == *found_namespace)
                .map(|(prefix, _)| prefix)
        })
    }

    /// This function takes in a namespace and tries to resolve it in different ways depending on the options provided. Unless `always_declare` is true, it will try to use an existing declaration. Otherwise, or if the namespace has not yet been declared, it will provide a declaration.
    pub fn resolve_namespace<'b>(
        &'b mut self,
        namespace: &'_ XmlNamespace<'b>,
        preferred_prefix: Option<&'b Prefix<'b>>,
        always_declare: IncludePrefix,
    ) -> (Prefix<'a>, Option<XmlnsDeclaration<'a>>) {
        if always_declare != IncludePrefix::Always {
            let existing_prefix = self.find_matching_namespace(namespace);

            if let Some(existing_prefix) = existing_prefix {
                return (existing_prefix.clone(), None);
            }
        }

        // If the namespace is not declared, use the specifically requested preferred prefix...
        // ...if it is not already used and not the same as the existing prefix.
        let prefix = preferred_prefix
            .filter(|p| self.get_namespace(p).is_none_or(|n| n == namespace))
            // If the preferred prefix is not available, use the preferred namespace prefix from the serializer...
            .or_else(|| {
                preferred_prefix
                    // ...if it is not already used and not the same as the existing prefix.
                    .filter(|p| self.get_namespace(p).is_none_or(|n| n == namespace))
            })
            .cloned()
            // If the preferred namespace prefix is not available, use a random prefix.
            .unwrap_or_else(|| self.prefix_generator.new_prefix())
            .into_owned();

        let scope = self
            .scopes
            .last_mut()
            .expect("There should be at least one scope");

        scope
            .defined_namespaces
            .insert(prefix.clone(), namespace.clone().into_owned());

        let (prefix, namespace) = scope
            .defined_namespaces
            .get_key_value(&prefix)
            .expect("The namespace should be defined as it was just added");

        let xmlns = XmlnsDeclaration::new(prefix.clone(), namespace.clone());

        (prefix.clone(), Some(xmlns))
    }

    pub fn resolve_name<'c>(
        &'c mut self,
        local_name: LocalName<'c>,
        namespace: &Option<XmlNamespace<'c>>,
        preferred_prefix: Option<&'c Prefix<'c>>,
        always_declare: IncludePrefix,
    ) -> (QName<'a>, Option<XmlnsDeclaration<'a>>) {
        let (prefix, declaration) = namespace
            .as_ref()
            .map(|namespace| self.resolve_namespace(namespace, preferred_prefix, always_declare))
            .unzip();

        let declaration = declaration.flatten();

        let name = QName::new(prefix, local_name.into_owned());
        (name, declaration)
    }
}

/// The [`xmlity::Deserializer`] for the `quick-xml` crate.
pub struct Serializer<W: Write> {
    writer: QuickXmlWriter<W>,
    preferred_namespace_prefixes: BTreeMap<XmlNamespace<'static>, Prefix<'static>>,
    namespace_scopes: NamespaceScopeContainer<'static>,
    buffered_bytes_start: BytesStart<'static>,
    buffered_bytes_start_empty: bool,
}

impl<W: Write> Serializer<W> {
    /// Create a new serializer.
    pub fn new(writer: QuickXmlWriter<W>) -> Self {
        Self::new_with_namespaces(writer, BTreeMap::new())
    }

    /// Create a new serializer with preferred namespace prefixes.
    pub fn new_with_namespaces(
        writer: QuickXmlWriter<W>,
        preferred_namespace_prefixes: BTreeMap<XmlNamespace<'static>, Prefix<'static>>,
    ) -> Self {
        Self {
            writer,
            preferred_namespace_prefixes,
            namespace_scopes: NamespaceScopeContainer::new(),
            buffered_bytes_start: BytesStart::new(""),
            buffered_bytes_start_empty: true,
        }
    }

    /// Consume the serializer and return the underlying writer.
    pub fn into_inner(self) -> W {
        self.writer.into_inner()
    }

    fn push_namespace_scope(&mut self) {
        self.namespace_scopes.push_scope()
    }

    fn pop_namespace_scope(&mut self) {
        self.namespace_scopes.pop_scope();
    }

    fn resolve_name<'b>(
        &mut self,
        name: ExpandedName<'b>,
        preferred_prefix: Option<&Prefix<'b>>,
        always_declare: IncludePrefix,
    ) -> (QName<'static>, Option<XmlnsDeclaration<'static>>) {
        let (local_name, namespace) = name.into_parts();

        let namespace_ref = namespace.as_ref();

        let preferred_prefix = preferred_prefix
            .or_else(|| namespace_ref.and_then(|a| self.preferred_namespace_prefixes.get(a)));

        self.namespace_scopes
            .resolve_name(local_name, &namespace, preferred_prefix, always_declare)
    }
}

impl<W: Write> From<QuickXmlWriter<W>> for Serializer<W> {
    fn from(writer: QuickXmlWriter<W>) -> Self {
        Self::new(writer)
    }
}

impl<W: Write> From<W> for Serializer<W> {
    fn from(writer: W) -> Self {
        Self::new(QuickXmlWriter::new(writer))
    }
}

/// The main element serializer for the `quick-xml` crate.
pub struct SerializeElement<'s, W: Write> {
    serializer: &'s mut Serializer<W>,
    name: ExpandedName<'static>,
    include_prefix: IncludePrefix,
    preferred_prefix: Option<Prefix<'static>>,
}

impl<'s, W: Write> SerializeElement<'s, W> {
    fn resolve_name_or_declare<'a>(
        name: ExpandedName<'a>,
        preferred_prefix: Option<&Prefix<'a>>,
        enforce_prefix: IncludePrefix,
        serializer: &mut Serializer<W>,
    ) -> (QName<'a>, Option<XmlnsDeclaration<'a>>) {
        let (qname, decl) = serializer.resolve_name(name, preferred_prefix, enforce_prefix);

        (qname, decl)
    }
}

/// The attribute serializer for the `quick-xml` crate.
pub struct AttributeSerializer<'t, W: Write> {
    name: ExpandedName<'static>,
    serializer: &'t mut Serializer<W>,
    preferred_prefix: Option<Prefix<'static>>,
    enforce_prefix: IncludePrefix,
}

impl<W: Write> ser::SerializeAttributeAccess for AttributeSerializer<'_, W> {
    type Ok = ();
    type Error = Error;

    fn include_prefix(&mut self, should_enforce: IncludePrefix) -> Result<Self::Ok, Self::Error> {
        self.enforce_prefix = should_enforce;
        Ok(())
    }

    fn preferred_prefix(
        &mut self,
        preferred_prefix: Option<xmlity::Prefix<'_>>,
    ) -> Result<Self::Ok, Self::Error> {
        self.preferred_prefix = preferred_prefix.map(Prefix::into_owned);
        Ok(())
    }

    fn end<S: AsRef<str>>(self, value: S) -> Result<Self::Ok, Self::Error> {
        let (qname, decl) = SerializeElement::resolve_name_or_declare(
            self.name,
            None,
            IncludePrefix::default(),
            self.serializer,
        );

        if let Some(decl) = decl {
            self.serializer.push_decl_attr(decl);
        }

        self.serializer.push_attr(qname, value.as_ref());

        Ok(())
    }
}

impl<'t, W: Write> ser::AttributeSerializer for &mut SerializeElementAttributes<'t, W> {
    type Error = Error;

    type Ok = ();
    type SerializeAttribute<'a>
        = AttributeSerializer<'a, W>
    where
        Self: 'a;

    fn serialize_attribute(
        &mut self,
        name: &'_ ExpandedName<'_>,
    ) -> Result<Self::SerializeAttribute<'_>, Self::Error> {
        Ok(Self::SerializeAttribute {
            name: name.clone().into_owned(),
            serializer: self.serializer.deref_mut(),
            preferred_prefix: None,
            enforce_prefix: IncludePrefix::default(),
        })
    }

    fn serialize_none(&mut self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'s, W: Write> SerializeElement<'s, W> {
    fn finish_start(self) -> (QName<'static>, &'s mut Serializer<W>) {
        let Self {
            name,
            include_prefix,
            preferred_prefix,
            serializer,
        } = self;

        assert!(
            serializer.buffered_bytes_start_empty,
            "Should have been emptied by the serializer"
        );

        serializer.buffered_bytes_start.clear_attributes();

        let (qname, decl) = SerializeElement::resolve_name_or_declare(
            name.clone(),
            preferred_prefix.as_ref(),
            include_prefix,
            serializer,
        );
        serializer
            .buffered_bytes_start
            .set_name(qname.to_string().as_bytes());

        if let Some(decl) = decl {
            serializer.push_decl_attr(decl);
        }
        serializer.buffered_bytes_start_empty = false;

        (qname, serializer)
    }

    fn end_empty(serializer: &mut Serializer<W>) -> Result<(), Error> {
        assert!(
            !serializer.buffered_bytes_start_empty,
            "start should be buffered"
        );
        let start = serializer.buffered_bytes_start.borrow();

        serializer
            .writer
            .write_event(Event::Empty(start))
            .map_err(Error::Io)?;

        serializer.buffered_bytes_start_empty = true;

        Ok(())
    }
}

/// Provides the implementation of [`ser::SerializeElement`] for the `quick-xml` crate.
pub struct SerializeElementAttributes<'s, W: Write> {
    serializer: &'s mut Serializer<W>,
    end_name: QName<'static>,
}

impl<W: Write> ser::SerializeAttributes for SerializeElementAttributes<'_, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_attribute<A: ser::SerializeAttribute>(
        &mut self,
        a: &A,
    ) -> Result<Self::Ok, Self::Error> {
        a.serialize_attribute(self)
    }
}

impl<'s, W: Write> ser::SerializeElementAttributes for SerializeElementAttributes<'s, W> {
    type ChildrenSerializeSeq = ChildrenSerializeSeq<'s, W>;

    fn serialize_children(self) -> Result<Self::ChildrenSerializeSeq, Self::Error> {
        Ok(ChildrenSerializeSeq {
            serializer: self.serializer,
            end_name: self.end_name,
        })
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeElement::end_empty(self.serializer)
    }
}

impl<'s, W: Write> ser::SerializeElement for SerializeElement<'s, W> {
    type Ok = ();
    type Error = Error;
    type ChildrenSerializeSeq = ChildrenSerializeSeq<'s, W>;
    type SerializeElementAttributes = SerializeElementAttributes<'s, W>;

    fn include_prefix(&mut self, should_enforce: IncludePrefix) -> Result<Self::Ok, Self::Error> {
        self.include_prefix = should_enforce;
        Ok(())
    }
    fn preferred_prefix(
        &mut self,
        preferred_prefix: Option<Prefix<'_>>,
    ) -> Result<Self::Ok, Self::Error> {
        self.preferred_prefix = preferred_prefix.map(Prefix::into_owned);
        Ok(())
    }

    fn serialize_attributes(self) -> Result<Self::SerializeElementAttributes, Self::Error> {
        self.serializer.push_namespace_scope();
        let (end_name, serializer) = self.finish_start();
        Ok(SerializeElementAttributes {
            serializer,
            end_name,
        })
    }

    fn serialize_children(self) -> Result<Self::ChildrenSerializeSeq, Self::Error> {
        self.serializer.push_namespace_scope();
        let (end_name, serializer) = self.finish_start();

        Ok(ChildrenSerializeSeq {
            serializer,
            end_name,
        })
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.serializer.push_namespace_scope();
        let (_, serializer) = self.finish_start();

        SerializeElement::end_empty(serializer)?;

        serializer.pop_namespace_scope();

        Ok(())
    }
}

///Provides the implementation of `SerializeSeq` trait for element children for the `quick-xml` crate.
pub struct ChildrenSerializeSeq<'s, W: Write> {
    serializer: &'s mut Serializer<W>,
    end_name: QName<'static>,
}

impl<W: Write> ser::SerializeSeq for ChildrenSerializeSeq<'_, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<V: Serialize>(&mut self, value: &V) -> Result<Self::Ok, Self::Error> {
        value.serialize(self.serializer.deref_mut())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // If we have a bytes_start, then we never wrote the start event, so we need to write an empty element instead.
        if !self.serializer.buffered_bytes_start_empty {
            self.serializer
                .writer
                .write_event(Event::Empty(self.serializer.buffered_bytes_start.borrow()))
                .map_err(Error::Io)?;
            self.serializer.buffered_bytes_start_empty = true;
        } else {
            let end_name = OwnedQuickName::new(&self.end_name);

            let bytes_end = BytesEnd::from(end_name.as_ref());

            self.serializer
                .writer
                .write_event(Event::End(bytes_end))
                .map_err(Error::Io)?;
        }

        self.serializer.pop_namespace_scope();

        Ok(())
    }
}

/// Provides the implementation of `SerializeSeq` trait for any nodes for the `quick-xml` crate.
pub struct SerializeSeq<'e, W: Write> {
    serializer: &'e mut Serializer<W>,
}

impl<W: Write> ser::SerializeSeq for SerializeSeq<'_, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<V: Serialize>(&mut self, v: &V) -> Result<Self::Ok, Self::Error> {
        v.serialize(self.serializer.deref_mut())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<W: Write> Serializer<W> {
    fn try_start(&mut self) -> Result<(), Error> {
        if !self.buffered_bytes_start_empty {
            self.writer
                .write_event(Event::Start(self.buffered_bytes_start.borrow()))
                .map_err(Error::Io)?;
            self.buffered_bytes_start_empty = true;
        }
        Ok(())
    }

    fn push_attr(&mut self, qname: QName<'_>, value: &str) {
        self.buffered_bytes_start
            .push_attribute(quick_xml::events::attributes::Attribute {
                key: quick_xml::name::QName(qname.to_string().as_bytes()),
                value: Cow::Borrowed(value.as_bytes()),
            });
    }

    fn push_decl_attr(&mut self, decl: XmlnsDeclaration<'_>) {
        let XmlnsDeclaration { namespace, prefix } = decl;

        let key = XmlnsDeclaration::xmlns_qname(prefix);

        self.push_attr(key, namespace.as_str());
    }
}

impl<'s, W: Write> xmlity::Serializer for &'s mut Serializer<W> {
    type Ok = ();
    type Error = Error;
    type SerializeElement = SerializeElement<'s, W>;
    type SerializeSeq = SerializeSeq<'s, W>;

    fn serialize_cdata<S: AsRef<str>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        self.try_start()?;
        self.writer
            .write_event(Event::CData(BytesCData::new(text.as_ref())))
            .map_err(Error::Io)
    }

    fn serialize_text<S: AsRef<str>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        self.try_start()?;
        self.writer
            .write_event(Event::Text(BytesText::from_escaped(text.as_ref())))
            .map_err(Error::Io)
    }

    fn serialize_element<'a>(
        self,
        name: &'a ExpandedName<'a>,
    ) -> Result<Self::SerializeElement, Self::Error> {
        self.try_start()?;

        Ok(SerializeElement {
            serializer: self,
            name: name.clone().into_owned(),
            include_prefix: IncludePrefix::default(),
            preferred_prefix: None,
        })
    }

    fn serialize_seq(self) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SerializeSeq { serializer: self })
    }

    fn serialize_decl<S: AsRef<str>>(
        self,
        version: S,
        encoding: Option<S>,
        standalone: Option<S>,
    ) -> Result<Self::Ok, Self::Error> {
        self.try_start()?;
        self.writer
            .write_event(Event::Decl(BytesDecl::new(
                version.as_ref(),
                encoding.as_ref().map(|s| s.as_ref()),
                standalone.as_ref().map(|s| s.as_ref()),
            )))
            .map_err(Error::Io)
    }

    fn serialize_pi<S: AsRef<[u8]>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        self.try_start()?;
        self.writer
            .write_event(Event::PI(BytesPI::new(
                str::from_utf8(text.as_ref()).unwrap(),
            )))
            .map_err(Error::Io)
    }

    fn serialize_comment<S: AsRef<[u8]>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        self.try_start()?;
        self.writer
            .write_event(Event::Comment(BytesText::from_escaped(
                str::from_utf8(text.as_ref()).unwrap(),
            )))
            .map_err(Error::Io)
    }

    fn serialize_doctype<S: AsRef<[u8]>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        self.try_start()?;
        self.writer
            .write_event(Event::DocType(BytesText::from_escaped(
                str::from_utf8(text.as_ref()).unwrap(),
            )))
            .map_err(Error::Io)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
