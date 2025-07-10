//! This module contains the [`Deserialize`], [`Deserializer`] and [`DeserializationGroup`] traits and associated types.

use std::{
    error::Error as StdError,
    fmt::{Debug, Display},
};

use crate::{ExpandedName, Prefix, XmlNamespace};

/// A trait for errors that can be returned by a [`Deserializer`].
pub trait Error: Sized + StdError {
    /// Error for when a custom error occurs during deserialization.
    fn custom<T>(msg: T) -> Self
    where
        T: Display;

    /// Error for when a name is expected to be a certain value, but it is not.
    fn wrong_name(name: &ExpandedName<'_>, expected: &ExpandedName<'_>) -> Self;

    /// Error for when a type is expected to be a certain type, but it is not.
    fn unexpected_visit<T>(unexpected: Unexpected, expected: &T) -> Self;

    /// Error for when a field is missing.
    fn missing_field(field: &str) -> Self;

    /// Error for when a type has no possible variants to deserialize into.
    fn no_possible_variant(ident: &str) -> Self;

    /// Error for when a type is missing data that is required to deserialize it.
    fn missing_data() -> Self;

    /// Error for when a child cannot be identified, and ignoring it is not allowed.
    fn unknown_child() -> Self;

    /// Error for when a string is invalid for the type.
    fn invalid_string() -> Self;
}

/// An enum representing the unexpected type of data that was encountered.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Unexpected {
    /// A text node.
    #[error("text")]
    Text,
    /// A CDATA section.
    #[error("cdata")]
    CData,
    /// A sequence of XML values.
    #[error("sequence")]
    Seq,
    /// An element start.
    #[error("element start")]
    ElementStart,
    /// An element end.
    #[error("element end")]
    ElementEnd,
    /// An attribute.
    #[error("attribute")]
    Attribute,
    /// A comment.
    #[error("comment")]
    Comment,
    /// A declaration.
    #[error("declaration")]
    Decl,
    /// A processing instruction.
    #[error("processing instruction")]
    PI,
    /// A doctype.
    #[error("doctype")]
    DocType,
    /// End of file.
    #[error("eof")]
    Eof,
    /// Nothing.
    #[error("none")]
    None,
}

#[derive(Debug)]
pub enum Event<'a> {
    StartElementOpened {
        name: &'a ExpandedName<'static>,
    },
    StartElementAttribute {
        name: &'a ExpandedName<'static>,
        value: &'a str,
    },
    StartElementClosed,
    EndElement {
        name: &'a ExpandedName<'static>,
    },
    Text {
        text: &'a str,
    },
    CData {
        text: &'a str,
    },
    Comment {
        text: &'a str,
    },
    PI {
        target: &'a str,
        data: &'a str,
    },
    Decl {
        version: &'a str,
        encoding: Option<&'a str>,
        standalone: Option<&'a str>,
    },
    DocType {
        name: &'a str,
        public_id: Option<&'a str>,
        system_id: Option<&'a str>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorA;

/// A type that can be used to deserialize XML documents.
pub trait Deserializer<'de>: Sized {
    /// The error type that can be returned from the deserializer.
    type Error: Error;
}

/// Trait that lets you access the namespaces declared on an XML node.
pub trait DeserializeContext {
    /// Get the default namespace.
    fn default_namespace(&self) -> Option<XmlNamespace<'_>>;

    /// Resolve a prefix to a namespace.
    fn resolve_prefix(&self, prefix: Prefix<'_>) -> Option<XmlNamespace<'_>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EventControl {
    Advance(usize),
    Rewind(usize),
    Done(usize),
}

/// A builder for a deserialization group. When completed (through [`DeserializationGroupBuilder::finish`]), the builder is converted into the deserialization group type that initated the builder.
pub trait DeserializeBuilder<'de>: Sized {
    /// The type of the deserialization group that this builder builds when finished through [`DeserializationGroupBuilder::finish`].
    type Value;

    /// Returns true if the deserializer made progress
    fn read_events(
        &mut self,
        context: &dyn DeserializeContext,
        packets: &[Event<'de>],
    ) -> Result<EventControl, ErrorA> {
        let _ = context;
        let _ = packets;

        Ok(EventControl::Done(0))
    }

    /// This function is called after all attributes and elements have been contributed.
    fn finish(self) -> Result<Self::Value, ErrorA>;
}

/// A type that can be deserialized from a deserializer. This type has two methods: [`Deserialize::deserialize`] and [`Deserialize::deserialize_seq`]. The latter is used in cases where types can be constructed from multiple nodes, such as constructing a [`std::vec::Vec`] from multiple elements, or a [`std::string::String`] from multiple text nodes that are concatenated together.
///
/// To see the documentation for the derive macro, see [`xmlity_derive::Deserialize`].
pub trait Deserialize<'de>: Sized {
    type Builder: DeserializeBuilder<'de, Value = Self>;

    fn builder() -> Self::Builder;
}

/// A utility type for easier use of [`Deserialize`] trait without needing to specify the lifetime.
pub trait DeserializeOwned: for<'de> Deserialize<'de> {}
impl<T> DeserializeOwned for T where T: for<'de> Deserialize<'de> {}

enum BuildState<'de, T: DeserializeBuilder<'de>> {
    Waiting,
    Building { builder: T, read_count: usize },
    Finishing,
    Done(T::Value),
}

impl<'de, T: DeserializeBuilder<'de>> BuildState<'de, T> {
    fn finish(&mut self) -> Result<(), ErrorA> {
        if matches!(self, BuildState::Done(_)) {
            return Ok(());
        }

        let BuildState::Building { builder, .. } = core::mem::replace(self, BuildState::Finishing)
        else {
            panic!("Cannot finish a BuildState that is not Building");
        };

        *self = BuildState::Done(builder.finish()?);

        Ok(())
    }

    fn expect_finish(mut self) -> Result<T::Value, ErrorA> {
        self.finish()?;

        match self {
            BuildState::Done(value) => Ok(value),
            _ => unreachable!("After calling `finish`, the state must be Done"),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::LocalName;

    use super::*;

    impl<'de> Deserialize<'de> for String {
        type Builder = String;

        fn builder() -> Self::Builder {
            String::new()
        }
    }

    impl<'de> DeserializeBuilder<'de> for String {
        type Value = String;

        fn read_events(
            &mut self,
            _context: &dyn DeserializeContext,
            packets: &[Event<'de>],
        ) -> Result<EventControl, ErrorA> {
            for (i, packet) in packets.iter().enumerate() {
                match packet {
                    Event::Text { text } => {
                        self.push_str(text);
                    }
                    _ => return Ok(EventControl::Done(i)),
                }
            }
            Ok(EventControl::Advance(packets.len()))
        }

        fn finish(self) -> Result<Self::Value, ErrorA> {
            Ok(self)
        }
    }

    #[derive(Debug, PartialEq)]
    struct ElemA {
        text: String,
    }
    enum ElemABuilder<'de> {
        AwaitingStart,
        AwaitingEndStart,
        Content {
            text: BuildState<'de, <String as Deserialize<'de>>::Builder>,
        },
        Finished(ElemA),
    }

    impl<'de> Deserialize<'de> for ElemA {
        type Builder = ElemABuilder<'de>;

        fn builder() -> Self::Builder {
            ElemABuilder::AwaitingStart
        }
    }

    impl<'de> DeserializeBuilder<'de> for ElemABuilder<'de> {
        type Value = ElemA;

        fn read_events(
            &mut self,
            context: &dyn DeserializeContext,
            packets: &[Event<'de>],
        ) -> Result<EventControl, ErrorA> {
            let mut remaining_packets = packets;
            while let Some(packet) = remaining_packets.first() {
                match self {
                    ElemABuilder::AwaitingStart => {
                        if let Event::StartElementOpened { name } = packet {
                            if **name == ExpandedName::new(LocalName::new_dangerous("ElemA"), None)
                            {
                                remaining_packets = &remaining_packets[1..];
                                *self = ElemABuilder::AwaitingEndStart;
                            } else {
                                return Err(ErrorA);
                            }
                        } else {
                            return Err(ErrorA);
                        }
                    }
                    ElemABuilder::AwaitingEndStart => {
                        if let Event::StartElementClosed = packet {
                            remaining_packets = &remaining_packets[1..];
                            *self = ElemABuilder::Content {
                                text: BuildState::Building {
                                    builder: <String as Deserialize<'de>>::builder(),
                                    read_count: 0,
                                },
                            };
                        } else {
                            return Err(ErrorA);
                        }
                    }
                    ElemABuilder::Content { text } => {
                        if let Event::EndElement { name } = packet {
                            if **name == ExpandedName::new(LocalName::new_dangerous("ElemA"), None)
                            {
                                let text = std::mem::replace(text, BuildState::Finishing);

                                remaining_packets = &remaining_packets[1..];

                                *self = ElemABuilder::Finished(ElemA {
                                    text: text.expect_finish()?,
                                });

                                return Ok(EventControl::Done(
                                    packets.len() - remaining_packets.len(),
                                ));
                            } else {
                                return Err(ErrorA);
                            }
                        }

                        if let BuildState::Building { builder, .. } = text {
                            match builder.read_events(context, remaining_packets)? {
                                EventControl::Advance(adv) => {
                                    remaining_packets = &remaining_packets[adv..];
                                }
                                EventControl::Rewind(adv) => {
                                    // This will try to rewind the used packets. If the amount rewinded is more than the amount of packets used in this call, it will return rewind to the parent minus the amount of packets used in this call.
                                    let used_packets = packets.len() - remaining_packets.len();
                                    if adv > used_packets {
                                        return Ok(EventControl::Rewind(adv - used_packets));
                                    }

                                    remaining_packets = &packets[(used_packets - adv)..];
                                }
                                EventControl::Done(adv) => {
                                    remaining_packets = &remaining_packets[adv..];

                                    text.finish()?;
                                }
                            }
                        }
                    }
                    ElemABuilder::Finished { .. } => {
                        return Err(ErrorA);
                    }
                }
            }

            Ok(EventControl::Advance(packets.len()))
        }

        fn finish(self) -> Result<Self::Value, ErrorA> {
            match self {
                ElemABuilder::Finished(a) => Ok(a),
                _ => Err(ErrorA),
            }
        }
    }

    #[derive(Debug, PartialEq)]
    enum EnumB {
        VariantA(ElemA),
        VariantB(String),
    }

    enum EnumBBuilder<'de> {
        VariantA {
            read_count: usize,
            builder: <ElemA as Deserialize<'de>>::Builder,
        },
        VariantAEnd(ElemA),
        VariantB {
            read_count: usize,
            builder: <String as Deserialize<'de>>::Builder,
        },
        VariantBEnd(String),
        Finishing,
    }

    impl<'de> DeserializeBuilder<'de> for EnumBBuilder<'de> {
        type Value = EnumB;

        fn read_events(
            &mut self,
            context: &dyn DeserializeContext,
            packets: &[Event<'de>],
        ) -> Result<EventControl, ErrorA> {
            match self {
                EnumBBuilder::VariantA {
                    read_count,
                    builder,
                } => {
                    let Ok(control) = builder.read_events(context, packets) else {
                        let rewind = *read_count;
                        *self = EnumBBuilder::VariantB {
                            read_count: 0,
                            builder: String::builder(),
                        };
                        return Ok(EventControl::Rewind(rewind));
                    };

                    match control {
                        EventControl::Advance(advance) => {
                            *read_count += advance;
                            Ok(EventControl::Advance(advance))
                        }
                        EventControl::Rewind(rewind) => Ok(EventControl::Rewind(rewind)),
                        EventControl::Done(used) => {
                            let EnumBBuilder::VariantA { builder, .. } =
                                std::mem::replace(self, EnumBBuilder::Finishing)
                            else {
                                unreachable!()
                            };
                            *self = EnumBBuilder::VariantAEnd(builder.finish()?);
                            Ok(EventControl::Done(used))
                        }
                    }
                }
                EnumBBuilder::VariantB {
                    builder,
                    read_count,
                } => {
                    let Ok(control) = builder.read_events(context, packets) else {
                        return Err(ErrorA);
                    };

                    match control {
                        EventControl::Advance(advance) => {
                            *read_count += advance;
                            Ok(EventControl::Advance(advance))
                        }
                        EventControl::Rewind(rewind) => Ok(EventControl::Rewind(rewind)),
                        EventControl::Done(used) => {
                            let EnumBBuilder::VariantB { builder, .. } =
                                std::mem::replace(self, EnumBBuilder::Finishing)
                            else {
                                unreachable!()
                            };
                            *self = EnumBBuilder::VariantBEnd(builder.finish()?);
                            Ok(EventControl::Done(used))
                        }
                    }
                }
                EnumBBuilder::VariantAEnd(_) | EnumBBuilder::VariantBEnd(_) => {
                    Ok(EventControl::Done(0))
                }
                EnumBBuilder::Finishing => unreachable!(),
            }
        }

        fn finish(self) -> Result<Self::Value, ErrorA> {
            match self {
                EnumBBuilder::VariantAEnd(a) => Ok(EnumB::VariantA(a)),
                EnumBBuilder::VariantBEnd(b) => Ok(EnumB::VariantB(b)),
                EnumBBuilder::VariantA { builder, .. } => builder.finish().map(EnumB::VariantA),
                EnumBBuilder::VariantB { builder, .. } => builder.finish().map(EnumB::VariantB),
                EnumBBuilder::Finishing => unreachable!(),
            }
        }
    }
    impl<'de> Deserialize<'de> for EnumB {
        type Builder = EnumBBuilder<'de>;

        fn builder() -> Self::Builder {
            EnumBBuilder::VariantA {
                read_count: 0,
                builder: ElemA::builder(),
            }
        }
    }

    struct DevContext;

    impl DeserializeContext for DevContext {
        fn default_namespace(&self) -> Option<XmlNamespace<'_>> {
            None
        }

        fn resolve_prefix(&self, _prefix: Prefix<'_>) -> Option<XmlNamespace<'_>> {
            None
        }
    }

    #[test]
    fn text_deserialize() {
        let events = vec![Event::Text { text: "Hello" }];

        let mut text = String::builder();
        text.read_events(&DevContext, &events)
            .expect("Failed to read events");

        let val = text.finish().expect("Failed to finish deserialization");

        assert_eq!(val, "Hello");
    }

    #[test]
    fn elem_deserialize() {
        let name = ExpandedName::new(LocalName::new_dangerous("ElemA"), None);

        let events = vec![
            Event::StartElementOpened { name: &name },
            Event::StartElementClosed,
            Event::Text { text: "Hello" },
            Event::EndElement { name: &name },
        ];

        let mut builder = ElemA::builder();

        builder
            .read_events(&DevContext, &events)
            .expect("Failed to read events");

        let val = builder.finish().expect("Failed to finish deserialization");

        assert_eq!(
            val,
            ElemA {
                text: "Hello".to_string()
            }
        );
    }

    #[test]
    fn enum_deserialize() {
        let events = vec![Event::Text { text: "Hello" }];

        let mut builder = EnumB::builder();

        let ctrl = builder
            .read_events(&DevContext, &events)
            .expect("Failed to read events");

        assert_eq!(ctrl, EventControl::Rewind(0));

        let ctrl = builder
            .read_events(&DevContext, &events)
            .expect("Failed to read events");

        assert_eq!(ctrl, EventControl::Advance(1));

        let val = builder.finish().expect("Failed to finish deserialization");

        assert_eq!(val, EnumB::VariantB("Hello".to_string()));
    }
}
