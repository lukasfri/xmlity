use core::str;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::io::Write;
use std::ops::DerefMut;

use quick_xml::events::{BytesCData, BytesDecl, BytesEnd, BytesPI, BytesStart, BytesText, Event};
use quick_xml::writer::Writer as QuickXmlWriter;

use xmlity::ser::IncludePrefix;
use xmlity::{ser, ExpandedName, Prefix, QName, Serialize, XmlNamespace};

use crate::{declaration_into_attribute, Attribute, OwnedQuickName, XmlnsDeclaration};

use super::Error;

fn serializer_to_string<T>(serializer: QuickXmlWriter<Vec<u8>>, value: &T) -> Result<String, Error>
where
    T: Serialize,
{
    let mut serializer = Serializer::from(serializer);
    value.serialize(&mut serializer)?;
    let bytes = serializer.into_inner();

    String::from_utf8(bytes).map_err(Error::InvalidUtf8)
}

pub fn to_string<T>(value: &T) -> Result<String, Error>
where
    T: Serialize,
{
    serializer_to_string(QuickXmlWriter::new(Vec::new()), value)
}

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
        namespace: &'b XmlNamespace<'b>,
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
        namespace: &'b XmlNamespace<'b>,
        preferred_prefix: Option<&'b Prefix<'b>>,
        always_declare: IncludePrefix,
    ) -> (Prefix<'b>, Option<XmlnsDeclaration<'b>>) {
        let existing_prefix = self
            .find_matching_namespace(namespace)
            // If we should always declare, we simply pretend it's not declared yet.
            .filter(|_p| always_declare != IncludePrefix::Always);

        if let Some(existing_prefix) = existing_prefix {
            return (existing_prefix.clone(), None);
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
            .unwrap_or_else(|| self.prefix_generator.new_prefix());

        let xmlns = XmlnsDeclaration::new(prefix.clone(), namespace.clone());

        self.scopes.last_mut().map(|a| {
            a.defined_namespaces
                .insert(prefix.clone().into_owned(), namespace.clone().into_owned())
        });

        (prefix, Some(xmlns))
    }

    pub fn resolve_name<'b>(
        &mut self,
        name: ExpandedName<'b>,
        preferred_prefix: Option<&Prefix<'b>>,
        always_declare: IncludePrefix,
    ) -> (QName<'b>, Option<XmlnsDeclaration<'b>>) {
        let (prefix, declaration) = name
            .namespace()
            .map(|namespace| self.resolve_namespace(namespace, preferred_prefix, always_declare))
            .unzip();

        let declaration = declaration.flatten().map(|a| a.into_owned());
        let resolved_prefix = prefix.map(|a| a.into_owned());

        let name = name.to_q_name(resolved_prefix);
        (name.into_owned(), declaration)
    }
}

pub struct Serializer<W: Write> {
    writer: QuickXmlWriter<W>,
    preferred_namespace_prefixes: BTreeMap<XmlNamespace<'static>, Prefix<'static>>,
    namespace_scopes: NamespaceScopeContainer<'static>,
}

impl<W: Write> Serializer<W> {
    pub fn new(writer: QuickXmlWriter<W>) -> Self {
        Self::new_with_namespaces(writer, BTreeMap::new())
    }

    pub fn new_with_namespaces(
        writer: QuickXmlWriter<W>,
        preferred_namespace_prefixes: BTreeMap<XmlNamespace<'static>, Prefix<'static>>,
    ) -> Self {
        Self {
            writer,
            preferred_namespace_prefixes,
            namespace_scopes: NamespaceScopeContainer::new(),
        }
    }

    pub fn into_inner(self) -> W {
        self.writer.into_inner()
    }

    pub fn push_namespace_scope(&mut self) {
        self.namespace_scopes.push_scope()
    }

    pub fn pop_namespace_scope(&mut self) {
        self.namespace_scopes.pop_scope();
    }

    pub fn add_preferred_prefix(
        &mut self,
        namespace: XmlNamespace<'static>,
        prefix: Prefix<'static>,
    ) {
        self.preferred_namespace_prefixes.insert(namespace, prefix);
    }

    pub fn resolve_name<'b>(
        &mut self,
        name: ExpandedName<'b>,
        preferred_prefix: Option<&Prefix<'b>>,
        always_declare: IncludePrefix,
    ) -> (QName<'b>, Option<XmlnsDeclaration<'b>>) {
        let name2 = name.clone();
        let preferred_prefix = preferred_prefix.or_else(|| {
            name2
                .namespace()
                .and_then(|a| self.preferred_namespace_prefixes.get(a))
        });

        self.namespace_scopes
            .resolve_name(name, preferred_prefix, always_declare)
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

pub struct SerializeElement<'s, W: Write> {
    serializer: &'s mut Serializer<W>,
    name: ExpandedName<'static>,
    attributes: Vec<Attribute<'static>>,
    preferred_prefix: Option<Prefix<'static>>,
    enforce_prefix: IncludePrefix,
}

pub struct AttributeSerializer<'t> {
    name: ExpandedName<'static>,
    on_end_add_to: &'t mut Vec<Attribute<'static>>,
    preferred_prefix: Option<Prefix<'static>>,
    enforce_prefix: IncludePrefix,
}

impl ser::SerializeAttributeAccess for AttributeSerializer<'_> {
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
        self.on_end_add_to.push(Attribute {
            name: self.name.into_owned(),
            value: value.as_ref().to_owned(),
            preferred_prefix: self.preferred_prefix,
            enforce_prefix: self.enforce_prefix,
        });

        Ok(())
    }
}

pub struct AttributeVecSerializer<'t> {
    attributes: &'t mut Vec<Attribute<'static>>,
}

impl ser::AttributeSerializer for AttributeVecSerializer<'_> {
    type Error = Error;

    type Ok = ();
    type SerializeAttribute<'a>
        = AttributeSerializer<'a>
    where
        Self: 'a;

    fn serialize_attribute(
        &mut self,
        name: &'_ ExpandedName<'_>,
    ) -> Result<Self::SerializeAttribute<'_>, Self::Error> {
        Ok(Self::SerializeAttribute {
            name: name.clone().into_owned(),
            on_end_add_to: &mut self.attributes,
            preferred_prefix: None,
            enforce_prefix: IncludePrefix::default(),
        })
    }

    fn serialize_none(&mut self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

/// This exists to allow us to transfer the practically completed BytesStart out of `finish_start` to then become the `quick_xml` BytesStart which is only exists by references to data.
pub struct OwnedBytesStart {
    name: OwnedQuickName,
    attributes: Vec<(OwnedQuickName, Vec<u8>)>,
}

impl OwnedBytesStart {
    pub fn as_quick_xml(&self) -> BytesStart<'_> {
        BytesStart::from(self.name.as_ref()).with_attributes(self.attributes.iter().map(
            |(key, value)| quick_xml::events::attributes::Attribute {
                key: key.as_ref(),
                value: Cow::Borrowed(value),
            },
        ))
    }
}

impl<'s, W: Write> SerializeElement<'s, W> {
    fn finish_start(self) -> (OwnedBytesStart, QName<'static>, &'s mut Serializer<W>) {
        let Self {
            serializer,
            name,
            attributes,
            enforce_prefix,
            preferred_prefix,
        } = self;

        let mut resolve_name_or_declare =
            |name: &ExpandedName<'_>,
             preferred_prefix: Option<&Prefix<'_>>,
             enforce_prefix: IncludePrefix|
             -> (QName<'static>, Option<XmlnsDeclaration<'static>>) {
                let (qname, decl) =
                    serializer.resolve_name(name.clone(), preferred_prefix, enforce_prefix);

                (qname.into_owned(), decl.map(|a| a.into_owned()))
            };

        let (elem_qname, elem_name_decl) =
            resolve_name_or_declare(&name, preferred_prefix.as_ref(), enforce_prefix);

        let (attr_prefixes, attr_decls): (Vec<_>, Vec<_>) = attributes
            .iter()
            .map(|a| &a.name)
            .map(|name| resolve_name_or_declare(name, None, IncludePrefix::default()))
            .unzip();

        let decls = elem_name_decl
            .into_iter()
            .chain(attr_decls.into_iter().flatten())
            .collect::<Vec<_>>();

        // Add declared namespaces first
        let mut q_attributes = decls
            .iter()
            .map(|decl| declaration_into_attribute(decl.clone()))
            .map(|attr| {
                (
                    OwnedQuickName::new(&attr.name),
                    attr.value.as_bytes().to_owned(),
                )
            })
            .collect::<Vec<_>>();

        // Then add the attributes
        q_attributes.extend(
            attributes
                .into_iter()
                .zip(attr_prefixes)
                .map(|(attr, qname)| attr.resolve(qname.prefix().cloned()))
                .map(|attr| {
                    (
                        OwnedQuickName::new(&attr.name),
                        attr.value.as_bytes().to_owned(),
                    )
                }),
        );

        let bytes_start = OwnedBytesStart {
            name: OwnedQuickName::new(&elem_qname),
            attributes: q_attributes,
        };

        (bytes_start, elem_qname, serializer)
    }
}

impl<W: Write> ser::SerializeAttributes for SerializeElement<'_, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_attribute<A: ser::SerializeAttribute>(
        &mut self,
        a: &A,
    ) -> Result<Self::Ok, Self::Error> {
        a.serialize_attribute(AttributeVecSerializer {
            attributes: &mut self.attributes,
        })
    }
}

impl<'s, W: Write> ser::SerializeElement for SerializeElement<'s, W> {
    type SerializeElementChildren = SerializeElementChildren<'s, W>;

    fn include_prefix(&mut self, should_enforce: IncludePrefix) -> Result<Self::Ok, Self::Error> {
        self.enforce_prefix = should_enforce;
        Ok(())
    }
    fn preferred_prefix(
        &mut self,
        preferred_prefix: Option<Prefix<'_>>,
    ) -> Result<Self::Ok, Self::Error> {
        self.preferred_prefix = preferred_prefix.map(Prefix::into_owned);
        Ok(())
    }

    fn serialize_children(self) -> Result<Self::SerializeElementChildren, Self::Error> {
        self.serializer.push_namespace_scope();
        let (bytes_start, end_name, serializer) = self.finish_start();

        serializer
            .writer
            .write_event(Event::Start(bytes_start.as_quick_xml()))
            .map_err(Error::Io)?;

        Ok(SerializeElementChildren {
            serializer,
            end_name,
        })
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.serializer.push_namespace_scope();
        let (bytes_start, _, serializer) = self.finish_start();

        serializer
            .writer
            .write_event(Event::Empty(bytes_start.as_quick_xml()))
            .map_err(Error::Io)?;

        serializer.pop_namespace_scope();

        Ok(())
    }
}

pub struct SerializeElementChildren<'s, W: Write> {
    serializer: &'s mut Serializer<W>,
    end_name: QName<'static>,
}

impl<W: Write> ser::SerializeChildren for SerializeElementChildren<'_, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_child<V: Serialize>(&mut self, value: &V) -> Result<Self::Ok, Self::Error> {
        value.serialize(self.serializer.deref_mut())
    }
}

impl<W: Write> ser::SerializeElementChildren for SerializeElementChildren<'_, W> {
    fn end(self) -> Result<Self::Ok, Self::Error> {
        let end_name = OwnedQuickName::new(&self.end_name);

        let bytes_end = BytesEnd::from(end_name.as_ref());

        self.serializer
            .writer
            .write_event(Event::End(bytes_end))
            .map_err(Error::Io)?;

        self.serializer.pop_namespace_scope();

        Ok(())
    }
}

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

impl<'s, W: Write> xmlity::Serializer for &'s mut Serializer<W> {
    type Ok = ();
    type Error = Error;
    type SerializeElement = SerializeElement<'s, W>;
    type SerializeSeq = SerializeSeq<'s, W>;

    fn serialize_cdata<S: AsRef<str>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        self.writer
            .write_event(Event::CData(BytesCData::new(text.as_ref())))
            .map_err(Error::Io)
    }

    fn serialize_text<S: AsRef<str>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        self.writer
            .write_event(Event::Text(BytesText::from_escaped(text.as_ref())))
            .map_err(Error::Io)
    }

    fn serialize_element<'a>(
        self,
        name: &'a ExpandedName<'a>,
    ) -> Result<Self::SerializeElement, Self::Error> {
        Ok(SerializeElement {
            serializer: self,
            name: name.clone().into_owned(),
            attributes: Vec::new(),
            preferred_prefix: None,
            enforce_prefix: IncludePrefix::default(),
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
        self.writer
            .write_event(Event::Decl(BytesDecl::new(
                version.as_ref(),
                encoding.as_ref().map(|s| s.as_ref()),
                standalone.as_ref().map(|s| s.as_ref()),
            )))
            .map_err(Error::Io)
    }

    fn serialize_pi<S: AsRef<[u8]>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        self.writer
            .write_event(Event::PI(BytesPI::new(
                str::from_utf8(text.as_ref()).unwrap(),
            )))
            .map_err(Error::Io)
    }

    fn serialize_comment<S: AsRef<[u8]>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        self.writer
            .write_event(Event::Comment(BytesText::from_escaped(
                str::from_utf8(text.as_ref()).unwrap(),
            )))
            .map_err(Error::Io)
    }

    fn serialize_doctype<S: AsRef<[u8]>>(self, text: S) -> Result<Self::Ok, Self::Error> {
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
