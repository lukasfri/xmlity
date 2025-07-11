use crate::{
    de::{self, AttributesAccess, ElementAccess, Visitor},
    Deserialize, Deserializer, ExpandedName,
};

use super::*;

impl<'de> Deserializer<'de> for &'de XmlValue {
    type Error = XmlValueDeserializerError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            XmlValue::Text(xml_text) => xml_text.deserialize_any(visitor),
            XmlValue::CData(xml_cdata) => xml_cdata.deserialize_any(visitor),
            XmlValue::Element(xml_element) => xml_element.deserialize_any(visitor),
            XmlValue::Seq(xml_seq) => XmlSeqAccess::new(xml_seq).deserialize_any(visitor),
            XmlValue::PI(xml_pi) => xml_pi.deserialize_any(visitor),
            XmlValue::Decl(xml_decl) => xml_decl.deserialize_any(visitor),
            XmlValue::Comment(xml_comment) => xml_comment.deserialize_any(visitor),
            XmlValue::Doctype(xml_doctype) => xml_doctype.deserialize_any(visitor),
            XmlValue::None => visitor.visit_none(),
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            XmlValue::Text(xml_text) => xml_text.deserialize_seq(visitor),
            XmlValue::CData(xml_cdata) => xml_cdata.deserialize_seq(visitor),
            XmlValue::Element(xml_element) => xml_element.deserialize_seq(visitor),
            XmlValue::Seq(xml_seq) => XmlSeqAccess::new(xml_seq).deserialize_seq(visitor),
            XmlValue::PI(xml_pi) => xml_pi.deserialize_seq(visitor),
            XmlValue::Decl(xml_decl) => xml_decl.deserialize_seq(visitor),
            XmlValue::Comment(xml_comment) => xml_comment.deserialize_seq(visitor),
            XmlValue::Doctype(xml_doctype) => xml_doctype.deserialize_seq(visitor),
            XmlValue::None => visitor.visit_none(),
        }
    }
}

impl<'de> Deserializer<'de> for &'de XmlText {
    type Error = XmlValueDeserializerError;
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
        visitor.visit_seq(Some(self))
    }
}

impl<'de> Deserializer<'de> for Option<&'de XmlText> {
    type Error = XmlValueDeserializerError;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Some(xml_text) => xml_text.deserialize_any(visitor),
            None => visitor.visit_none(),
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Some(xml_text) => xml_text.deserialize_seq(visitor),
            None => visitor.visit_none(),
        }
    }
}

impl<'de> Deserializer<'de> for &'de XmlCData {
    type Error = XmlValueDeserializerError;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_cdata(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

impl<'de> de::XmlCData<'de> for &'de XmlCData {
    type DeserializeContext<'a>
        = ()
    where
        Self: 'a;

    fn into_bytes(self) -> Cow<'de, [u8]> {
        Cow::Borrowed(&self.0)
    }

    fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    fn into_string(self) -> Cow<'de, str> {
        Cow::Borrowed(std::str::from_utf8(&self.0).unwrap())
    }

    fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).unwrap()
    }

    fn context(&self) -> Self::DeserializeContext<'_> {}
}

impl<'de> Deserializer<'de> for &'de XmlChild {
    type Error = XmlValueDeserializerError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            XmlChild::Text(xml_text) => xml_text.deserialize_any(visitor),
            XmlChild::CData(xml_cdata) => xml_cdata.deserialize_any(visitor),
            XmlChild::Element(xml_element) => xml_element.deserialize_any(visitor),
            XmlChild::PI(xml_pi) => xml_pi.deserialize_any(visitor),
            XmlChild::Comment(xml_comment) => xml_comment.deserialize_any(visitor),
            XmlChild::None => visitor.visit_none(),
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            XmlChild::Text(xml_text) => xml_text.deserialize_seq(visitor),
            XmlChild::CData(xml_cdata) => xml_cdata.deserialize_seq(visitor),
            XmlChild::Element(xml_element) => xml_element.deserialize_seq(visitor),
            XmlChild::PI(xml_pi) => xml_pi.deserialize_seq(visitor),
            XmlChild::Comment(xml_comment) => xml_comment.deserialize_seq(visitor),
            XmlChild::None => visitor.visit_none(),
        }
    }
}

impl<'de> Deserializer<'de> for &'de XmlElement {
    type Error = XmlValueDeserializerError;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_element(XmlElementAccess {
            element: self,
            attribute_index: 0,
            write_attribute_index_to: None,
        })
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

impl<'de> crate::de::Deserializer<'de> for &'de XmlAttribute {
    type Error = XmlValueDeserializerError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_attribute(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_attribute(self)
    }
}

impl<'de> Deserializer<'de> for &mut XmlSeqAccess<'de, '_, XmlValue> {
    type Error = XmlValueDeserializerError;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

impl<'de> Deserializer<'de> for &mut XmlSeqAccess<'de, '_, XmlChild> {
    type Error = XmlValueDeserializerError;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

impl<'de> Deserializer<'de> for &'de XmlSeq<XmlValue> {
    type Error = XmlValueDeserializerError;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(XmlSeqAccess::new(self))
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

impl<'de> Deserializer<'de> for &'de XmlSeq<XmlChild> {
    type Error = XmlValueDeserializerError;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(XmlSeqAccess::new(self))
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

impl<'de> Deserializer<'de> for &'de XmlProcessingInstruction {
    type Error = XmlValueDeserializerError;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_pi(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

impl<'de> Deserializer<'de> for &'de XmlDecl {
    type Error = XmlValueDeserializerError;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_decl(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

impl<'de> Deserializer<'de> for &'de XmlComment {
    type Error = XmlValueDeserializerError;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_comment(self)
    }
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

impl<'de> Deserializer<'de> for &'de XmlDoctype {
    type Error = XmlValueDeserializerError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_doctype(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

// Element access

struct XmlElementAccess<'de, 'i> {
    element: &'de XmlElement,
    attribute_index: usize,
    write_attribute_index_to: Option<&'i mut usize>,
}

impl Drop for XmlElementAccess<'_, '_> {
    fn drop(&mut self) {
        if let Some(write_to) = self.write_attribute_index_to.as_mut() {
            **write_to = self.attribute_index;
        }
    }
}

impl<'de> AttributesAccess<'de> for XmlElementAccess<'de, '_> {
    type Error = XmlValueDeserializerError;

    type SubAccess<'a>
        = XmlElementAccess<'de, 'a>
    where
        Self: 'a;

    fn next_attribute<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'de>,
    {
        let Some(attribute) = self.element.attributes.get(self.attribute_index) else {
            return Ok(None);
        };
        let attribute = T::deserialize(attribute)?;
        self.attribute_index += 1;
        Ok(Some(attribute))
    }

    fn sub_access(&mut self) -> Result<Self::SubAccess<'_>, Self::Error> {
        Ok(XmlElementAccess {
            attribute_index: self.attribute_index,
            element: self.element,
            write_attribute_index_to: Some(&mut self.attribute_index),
        })
    }
}

impl<'de> ElementAccess<'de> for XmlElementAccess<'de, '_> {
    type ChildrenAccess = XmlSeqAccess<'de, 'static, XmlChild>;
    type DeserializeContext<'a>
        = ()
    where
        Self: 'a;

    fn name(&self) -> ExpandedName<'_> {
        self.element.name.clone()
    }

    fn children(self) -> Result<Self::ChildrenAccess, Self::Error> {
        Ok(XmlSeqAccess {
            seq: &self.element.children,
            index: 0,
            write_index_to: None,
        })
    }

    fn context(&self) -> Self::DeserializeContext<'_> {}
}

// Seq

struct XmlSeqAccess<'de, 'i, T> {
    seq: &'de XmlSeq<T>,
    index: usize,
    write_index_to: Option<&'i mut usize>,
}

impl<'de, T> XmlSeqAccess<'de, '_, T> {
    pub fn new(seq: &'de XmlSeq<T>) -> Self {
        Self {
            seq,
            index: 0,
            write_index_to: None,
        }
    }
}

impl<T> Drop for XmlSeqAccess<'_, '_, T> {
    fn drop(&mut self) {
        if let Some(write_index_to) = self.write_index_to.as_mut() {
            **write_index_to = self.index;
        }
    }
}

// One would think that these impls for XmlSeqAccess that take XmlValue and XmlChild should be unified using a generic impl, but this does not appear to be possible due to an error mentioning limits in the borrow checker.
// I've fought the borrow checker for a long time and lost, so for now, these are separate impls.
// If you want to take a stab at unifying these, be my guest.
impl<'de> de::SeqAccess<'de> for XmlSeqAccess<'de, '_, XmlChild> {
    type Error = XmlValueDeserializerError;
    type SubAccess<'g>
        = XmlSeqAccess<'de, 'g, XmlChild>
    where
        Self: 'g;
    fn next_element<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'de>,
    {
        let Some(value) = self.seq.values.get(self.index) else {
            return Ok(None);
        };
        let value = T::deserialize(value)?;
        self.index += 1;
        Ok(Some(value))
    }

    fn next_element_seq<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'de>,
    {
        T::deserialize_seq(self).map(Some)
    }

    fn sub_access(&mut self) -> Result<Self::SubAccess<'_>, Self::Error> {
        Ok(XmlSeqAccess {
            seq: self.seq,
            index: self.index,
            write_index_to: Some(&mut self.index),
        })
    }
}

impl<'de> de::SeqAccess<'de> for XmlSeqAccess<'de, '_, XmlValue> {
    type Error = XmlValueDeserializerError;
    type SubAccess<'g>
        = XmlSeqAccess<'de, 'g, XmlValue>
    where
        Self: 'g;
    fn next_element<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'de>,
    {
        let Some(value) = self.seq.values.get(self.index) else {
            return Ok(None);
        };
        let value = T::deserialize(value)?;
        self.index += 1;
        Ok(Some(value))
    }

    fn next_element_seq<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'de>,
    {
        T::deserialize_seq(self).map(Some)
    }

    fn sub_access(&mut self) -> Result<Self::SubAccess<'_>, Self::Error> {
        Ok(XmlSeqAccess {
            seq: self.seq,
            index: self.index,
            write_index_to: Some(&mut self.index),
        })
    }
}

impl de::XmlProcessingInstruction for &XmlProcessingInstruction {
    type DeserializeContext<'a>
        = ()
    where
        Self: 'a;

    fn content(&self) -> &[u8] {
        self.content.as_slice()
    }

    fn target(&self) -> &[u8] {
        self.target.as_slice()
    }
    fn context(&self) -> Self::DeserializeContext<'_> {}
}

impl de::XmlDeclaration for &XmlDecl {
    type DeserializeContext<'a>
        = ()
    where
        Self: 'a;

    fn version(&self) -> &[u8] {
        self.version.as_bytes()
    }

    fn encoding(&self) -> Option<&[u8]> {
        self.encoding.as_deref().map(|e| e.as_bytes())
    }

    fn standalone(&self) -> Option<&[u8]> {
        self.standalone.as_deref().map(|s| s.as_bytes())
    }

    fn context(&self) -> Self::DeserializeContext<'_> {}
}

impl<'de> de::XmlComment<'de> for &'de XmlComment {
    type DeserializeContext<'a>
        = ()
    where
        Self: 'a;

    fn into_bytes(self) -> Cow<'de, [u8]> {
        Cow::Borrowed(self.0.as_slice())
    }

    fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    fn context(&self) -> Self::DeserializeContext<'_> {}
}

impl<'de> de::XmlDoctype<'de> for &'de XmlDoctype {
    type DeserializeContext<'a>
        = ()
    where
        Self: 'a;

    fn into_bytes(self) -> Cow<'de, [u8]> {
        Cow::Borrowed(self.0.as_slice())
    }

    fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    fn context(&self) -> Self::DeserializeContext<'_> {}
}

impl<'de> de::XmlText<'de> for &'de XmlText {
    type DeserializeContext<'a>
        = ()
    where
        Self: 'a;

    fn into_bytes(self) -> Cow<'de, [u8]> {
        Cow::Borrowed(&self.0)
    }

    fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    fn into_string(self) -> Cow<'de, str> {
        Cow::Borrowed(std::str::from_utf8(&self.0).unwrap())
    }

    fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).unwrap()
    }

    fn context(&self) -> Self::DeserializeContext<'_> {}
}

impl<'de> de::SeqAccess<'de> for Option<&'de XmlText> {
    type Error = XmlValueDeserializerError;

    type SubAccess<'g>
        = Self
    where
        Self: 'g;

    fn next_element<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'de>,
    {
        let Some(text) = self.take() else {
            return Ok(None);
        };

        match T::deserialize(text) {
            Ok(value) => Ok(Some(value)),
            Err(_) => {
                *self = Some(text);
                Ok(None)
            }
        }
    }

    fn next_element_seq<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'de>,
    {
        let Some(text) = self.take() else {
            return Ok(None);
        };

        match T::deserialize_seq(text) {
            Ok(value) => Ok(Some(value)),
            Err(_) => {
                *self = Some(text);
                Ok(None)
            }
        }
    }

    fn sub_access(&mut self) -> Result<Self::SubAccess<'_>, Self::Error> {
        Ok(*self)
    }
}
