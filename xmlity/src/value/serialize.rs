use crate::{
    ser::{
        SerializeAttributeAccess as _, SerializeAttributes as _, SerializeElement as _,
        SerializeElementAttributes as _, SerializeSeq as _,
    },
    AttributeSerializer, Serialize, SerializeAttribute, Serializer,
};

use super::*;

impl Serialize for XmlValue {
    fn serialize<S: crate::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            XmlValue::Text(xml_text) => xml_text.serialize(serializer),
            XmlValue::CData(xml_cdata) => xml_cdata.serialize(serializer),
            XmlValue::Element(xml_element) => xml_element.serialize(serializer),
            XmlValue::Seq(xml_seq) => xml_seq.serialize(serializer),
            XmlValue::PI(xml_pi) => xml_pi.serialize(serializer),
            XmlValue::Decl(xml_decl) => xml_decl.serialize(serializer),
            XmlValue::Comment(xml_comment) => xml_comment.serialize(serializer),
            XmlValue::Doctype(xml_doctype) => xml_doctype.serialize(serializer),
            XmlValue::None => serializer.serialize_none(),
        }
    }
}
impl Serialize for XmlText {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_text(String::from_utf8_lossy(&self.0))
    }
}
impl Serialize for XmlCData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_cdata(str::from_utf8(&self.0).unwrap())
    }
}

impl Serialize for XmlChild {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            XmlChild::Text(v) => v.serialize(serializer),
            XmlChild::CData(v) => v.serialize(serializer),
            XmlChild::Element(v) => v.serialize(serializer),
            XmlChild::PI(v) => v.serialize(serializer),
            XmlChild::Comment(v) => v.serialize(serializer),
            XmlChild::None => serializer.serialize_none(),
        }
    }
}
impl Serialize for XmlElement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let element = serializer.serialize_element(&self.name.as_ref())?;

        let mut attributes = element.serialize_attributes()?;
        for attr in &self.attributes {
            attributes.serialize_attribute(attr)?;
        }

        if self.children.values.is_empty() {
            return attributes.end();
        }

        let mut children = attributes.serialize_children()?;
        for child in &self.children.values {
            children.serialize_element(child)?;
        }
        children.end()
    }
}
impl<T: Serialize> Serialize for XmlSeq<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: crate::ser::Serializer,
    {
        let mut seq = serializer.serialize_seq()?;
        for item in self.values.iter() {
            seq.serialize_element(item)?;
        }

        seq.end()
    }
}

impl Serialize for XmlProcessingInstruction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: crate::ser::Serializer,
    {
        serializer.serialize_pi(&self.target, &self.content)
    }
}

impl Serialize for XmlDecl {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_decl(
            self.version.deref(),
            self.encoding.as_deref(),
            self.standalone.as_deref(),
        )
    }
}

impl Serialize for XmlComment {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_comment(&self.0)
    }
}

impl Serialize for XmlDoctype {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_doctype(&self.0)
    }
}

impl SerializeAttribute for XmlAttribute {
    fn serialize_attribute<S>(&self, mut serializer: S) -> Result<S::Ok, S::Error>
    where
        S: AttributeSerializer,
    {
        let attr = serializer.serialize_attribute(&self.name.as_ref())?;

        attr.end(&self.value)
    }
}
