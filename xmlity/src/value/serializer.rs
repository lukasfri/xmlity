use crate::{
    noop::NoopDeSerializer,
    ser::{self, Error, IncludePrefix, SerializeAttributeAccess, Unexpected},
    ExpandedName, Prefix, Serialize, SerializeAttribute, Serializer,
};

use super::*;

impl<'s> Serializer for &'s mut &mut XmlSeq<XmlValue> {
    type Ok = ();
    type Error = XmlValueSerializerError;

    type SerializeSeq = &'s mut XmlSeq<XmlValue>;
    type SerializeElement = &'s mut XmlElement;

    fn serialize_cdata<S: AsRef<str>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        self.values
            .push_back(XmlValue::CData(XmlCData::new(text.as_ref().as_bytes())));
        Ok(())
    }

    fn serialize_text<S: AsRef<str>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        self.values.push_back(XmlValue::Text(XmlText::new(text)));
        Ok(())
    }

    fn serialize_element(
        self,
        name: &'_ ExpandedName<'_>,
    ) -> Result<Self::SerializeElement, Self::Error> {
        self.values.push_back(XmlValue::Element(XmlElement::new(
            name.clone().into_owned(),
        )));

        let XmlValue::Element(element) = self.values.back_mut().expect("just push_backed") else {
            unreachable!()
        };

        Ok(element)
    }

    fn serialize_seq(self) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(self)
    }

    fn serialize_pi<S: AsRef<[u8]>>(self, target: S, content: S) -> Result<Self::Ok, Self::Error> {
        self.values
            .push_back(XmlValue::PI(XmlProcessingInstruction {
                target: target.as_ref().to_vec(),
                content: content.as_ref().to_vec(),
            }));
        Ok(())
    }

    fn serialize_comment<S: AsRef<[u8]>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        self.values
            .push_back(XmlValue::Comment(XmlComment(text.as_ref().to_vec())));
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.values.push_back(XmlValue::None);
        Ok(())
    }

    fn serialize_decl<S: AsRef<str>>(
        self,
        version: S,
        encoding: Option<S>,
        standalone: Option<S>,
    ) -> Result<Self::Ok, Self::Error> {
        self.values
            .push_back(XmlValue::Decl(XmlDecl::new(version, encoding, standalone)));
        Ok(())
    }

    fn serialize_doctype<S: AsRef<[u8]>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        self.values
            .push_back(XmlValue::Doctype(XmlDoctype(text.as_ref().to_vec())));
        Ok(())
    }
}

impl<'s> Serializer for &'s mut XmlValue {
    type Ok = ();
    type Error = XmlValueSerializerError;

    type SerializeSeq = &'s mut XmlSeq<XmlValue>;
    type SerializeElement = &'s mut XmlElement;

    fn serialize_cdata<S: AsRef<str>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        *self = XmlValue::CData(XmlCData::new(text.as_ref().as_bytes()));
        Ok(())
    }

    fn serialize_text<S: AsRef<str>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        *self = XmlValue::Text(XmlText::new(text));
        Ok(())
    }

    fn serialize_element(
        self,
        name: &'_ ExpandedName<'_>,
    ) -> Result<Self::SerializeElement, Self::Error> {
        *self = XmlValue::Element(XmlElement::new(name.clone().into_owned()));

        let XmlValue::Element(element) = self else {
            unreachable!()
        };

        Ok(element)
    }

    fn serialize_seq(self) -> Result<Self::SerializeSeq, Self::Error> {
        *self = XmlValue::Seq(XmlSeq::new());
        let XmlValue::Seq(seq) = self else {
            unreachable!()
        };
        Ok(seq)
    }

    fn serialize_decl<S: AsRef<str>>(
        self,
        version: S,
        encoding: Option<S>,
        standalone: Option<S>,
    ) -> Result<Self::Ok, Self::Error> {
        *self = XmlValue::Decl(XmlDecl::new(version, encoding, standalone));
        Ok(())
    }

    fn serialize_pi<S: AsRef<[u8]>>(self, target: S, content: S) -> Result<Self::Ok, Self::Error> {
        *self = XmlValue::PI(XmlProcessingInstruction {
            target: target.as_ref().to_vec(),
            content: content.as_ref().to_vec(),
        });
        Ok(())
    }

    fn serialize_comment<S: AsRef<[u8]>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        *self = XmlValue::Comment(XmlComment(text.as_ref().to_vec()));
        Ok(())
    }

    fn serialize_doctype<S: AsRef<[u8]>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        *self = XmlValue::Doctype(XmlDoctype(text.as_ref().to_vec()));
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        *self = XmlValue::None;
        Ok(())
    }
}

impl<'s> Serializer for &'s mut &mut XmlSeq<XmlChild> {
    type Ok = ();
    type Error = XmlValueSerializerError;

    type SerializeSeq = &'s mut XmlSeq<XmlChild>;
    type SerializeElement = &'s mut XmlElement;

    fn serialize_cdata<S: AsRef<str>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        self.values
            .push_back(XmlChild::CData(XmlCData::new(text.as_ref().as_bytes())));
        Ok(())
    }

    fn serialize_text<S: AsRef<str>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        self.values.push_back(XmlChild::Text(XmlText::new(text)));
        Ok(())
    }

    fn serialize_element(
        self,
        name: &'_ ExpandedName<'_>,
    ) -> Result<Self::SerializeElement, Self::Error> {
        self.values.push_back(XmlChild::Element(XmlElement::new(
            name.clone().into_owned(),
        )));

        let XmlChild::Element(element) = self.values.back_mut().expect("just push_backed") else {
            unreachable!()
        };

        Ok(element)
    }

    fn serialize_seq(self) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(self)
    }

    fn serialize_pi<S: AsRef<[u8]>>(self, target: S, content: S) -> Result<Self::Ok, Self::Error> {
        self.values
            .push_back(XmlChild::PI(XmlProcessingInstruction {
                target: target.as_ref().to_vec(),
                content: content.as_ref().to_vec(),
            }));
        Ok(())
    }

    fn serialize_comment<S: AsRef<[u8]>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        self.values
            .push_back(XmlChild::Comment(XmlComment(text.as_ref().to_vec())));
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.values.push_back(XmlChild::None);
        Ok(())
    }

    fn serialize_decl<S: AsRef<str>>(
        self,
        _version: S,
        _encoding: Option<S>,
        _standalone: Option<S>,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Error::unexpected_serialize(Unexpected::Decl))
    }

    fn serialize_doctype<S: AsRef<[u8]>>(self, _text: S) -> Result<Self::Ok, Self::Error> {
        Err(Error::unexpected_serialize(Unexpected::DocType))
    }
}

impl ser::SerializeAttributes for &mut XmlElement {
    type Ok = ();

    type Error = XmlValueSerializerError;

    fn serialize_attribute<A: SerializeAttribute>(
        &mut self,
        a: &A,
    ) -> Result<Self::Ok, Self::Error> {
        a.serialize_attribute(self)?;

        Ok(())
    }
}

/// Builder used when serializing to an [`XmlAttribute``].
pub struct XmlAttributeBuilder<'a> {
    name: ExpandedName<'static>,
    write_to: &'a mut VecDeque<XmlAttribute>,
    should_enforce: IncludePrefix,
    preferred_prefix: Option<Prefix<'static>>,
}

impl<'a> XmlAttributeBuilder<'a> {
    /// Creates a new [`XmlAttributeBuilder`].
    pub fn new(name: ExpandedName<'static>, write_to: &'a mut VecDeque<XmlAttribute>) -> Self {
        Self {
            name,
            write_to,
            should_enforce: IncludePrefix::default(),
            preferred_prefix: None,
        }
    }
}

impl ser::AttributeSerializer for &mut &mut XmlElement {
    type Ok = ();
    type Error = XmlValueSerializerError;

    type SerializeAttribute<'a>
        = XmlAttributeBuilder<'a>
    where
        Self: 'a;

    fn serialize_attribute(
        &mut self,
        name: &'_ ExpandedName<'_>,
    ) -> Result<Self::SerializeAttribute<'_>, Self::Error> {
        Ok(XmlAttributeBuilder {
            name: name.clone().into_owned(),
            write_to: &mut self.attributes,
            should_enforce: IncludePrefix::default(),
            preferred_prefix: None,
        })
    }

    fn serialize_none(&mut self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'s> ser::SerializeElementAttributes for &'s mut XmlElement {
    type ChildrenSerializeSeq = &'s mut XmlSeq<XmlChild>;

    fn serialize_children(self) -> Result<Self::ChildrenSerializeSeq, Self::Error> {
        Ok(&mut self.children)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'s> ser::SerializeElement for &'s mut XmlElement {
    type Ok = ();

    type Error = XmlValueSerializerError;

    type ChildrenSerializeSeq = &'s mut XmlSeq<XmlChild>;
    type SerializeElementAttributes = &'s mut XmlElement;

    fn include_prefix(&mut self, should_enforce: IncludePrefix) -> Result<Self::Ok, Self::Error> {
        self.enforce_prefix = should_enforce;
        Ok(())
    }

    fn preferred_prefix(
        &mut self,
        preferred_prefix: Option<crate::Prefix<'_>>,
    ) -> Result<Self::Ok, Self::Error> {
        self.preferred_prefix = preferred_prefix.map(Prefix::into_owned);
        Ok(())
    }

    fn serialize_children(self) -> Result<Self::ChildrenSerializeSeq, Self::Error> {
        Ok(&mut self.children)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_attributes(self) -> Result<Self::SerializeElementAttributes, Self::Error> {
        Ok(self)
    }
}

// Seq

impl crate::ser::SerializeSeq for &mut XmlSeq<XmlValue> {
    type Ok = ();

    type Error = XmlValueSerializerError;

    fn serialize_element<V: Serialize>(&mut self, v: &V) -> Result<(), Self::Error> {
        v.serialize(self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl crate::ser::SerializeSeq for &mut XmlSeq<XmlChild> {
    type Ok = ();

    type Error = XmlValueSerializerError;

    fn serialize_element<V: Serialize>(&mut self, v: &V) -> Result<(), Self::Error> {
        v.serialize(self)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl SerializeAttributeAccess for XmlAttributeBuilder<'_> {
    type Ok = ();

    type Error = XmlValueSerializerError;

    fn include_prefix(&mut self, should_enforce: IncludePrefix) -> Result<Self::Ok, Self::Error> {
        self.should_enforce = should_enforce;
        Ok(())
    }

    fn preferred_prefix(
        &mut self,
        preferred_prefix: Option<crate::Prefix<'_>>,
    ) -> Result<Self::Ok, Self::Error> {
        self.preferred_prefix = preferred_prefix.map(|p| p.into_owned());
        Ok(())
    }

    /// Serialize the attribute.
    fn end<S: Serialize>(self, value: &S) -> Result<Self::Ok, Self::Error> {
        let mut value_container = XmlText::new("");
        value.serialize(&mut value_container)?;
        self.write_to.push_back(XmlAttribute {
            name: self.name,
            value: value_container,
        });
        Ok(())
    }
}

impl crate::ser::Serializer for &mut XmlText {
    type Ok = ();

    type Error = XmlValueSerializerError;

    type SerializeElement = NoopDeSerializer<Self::Ok, XmlValueSerializerError>;

    type SerializeSeq = NoopDeSerializer<Self::Ok, XmlValueSerializerError>;

    fn serialize_text<S: AsRef<str>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        self.0 = text.as_ref().as_bytes().to_vec();

        Ok(())
    }

    fn serialize_cdata<S: AsRef<str>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        let _ = text;

        Err(Error::unexpected_serialize(Unexpected::CData))
    }

    fn serialize_element(
        self,
        name: &'_ ExpandedName<'_>,
    ) -> Result<Self::SerializeElement, Self::Error> {
        let _ = name;

        Err(Error::unexpected_serialize(Unexpected::Element))
    }

    fn serialize_seq(self) -> Result<Self::SerializeSeq, Self::Error> {
        Err(Error::unexpected_serialize(Unexpected::Seq))
    }

    fn serialize_decl<S: AsRef<str>>(
        self,
        version: S,
        encoding: Option<S>,
        standalone: Option<S>,
    ) -> Result<Self::Ok, Self::Error> {
        let _ = (version, encoding, standalone);

        Err(Error::unexpected_serialize(Unexpected::Decl))
    }

    fn serialize_pi<S: AsRef<[u8]>>(self, target: S, content: S) -> Result<Self::Ok, Self::Error> {
        let _ = (target, content);

        Err(Error::unexpected_serialize(Unexpected::PI))
    }

    fn serialize_comment<S: AsRef<[u8]>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        let _ = text;

        Err(Error::unexpected_serialize(Unexpected::Comment))
    }

    fn serialize_doctype<S: AsRef<[u8]>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        let _ = text;

        Err(Error::unexpected_serialize(Unexpected::DocType))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::unexpected_serialize(Unexpected::None))
    }
}
