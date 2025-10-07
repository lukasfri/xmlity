use super::*;

pub mod roots {
    use syn::{parse_quote, Path};

    use super::*;

    #[derive(FromAttributes, Default)]
    #[darling(attributes(xvalue))]
    pub struct RootValueOpts {
        /// The text casing to use for unit variants when serializing and deserializing.
        #[darling(default)]
        pub rename_all: RenameRule,
        /// The path to the module that provides the serialization and deserialization functions.
        ///
        /// `::serialize` and `::deserialize` will be appended to this path and used as the `serialize_with` and `deserialize_with` functions.
        #[darling(default)]
        pub with: Option<Path>,
        /// Use function to serialize the value.
        ///
        /// Should have signature like `pub fn serialize<S: xmlity::Serializer>(value: &T, serializer: S) -> Result<S::Ok, S::Error>`
        #[darling(default)]
        pub serialize_with: Option<Expr>,
        /// Use function to deserialize the value.
        ///
        /// Should have signature like `fn deserialize<'de, D: xmlity::Deserializer<'de>>(deserializer: D) -> Result<T, D::Error>`
        #[darling(default)]
        pub deserialize_with: Option<Expr>,
    }

    impl RootValueOpts {
        pub fn parse(ast: &DeriveInput) -> Result<Option<Self>, DeriveError> {
            let Some(attr) = ast.attrs.iter().find(|attr| attr.path().is_ident("xvalue")) else {
                return Ok(None);
            };

            let opts = Self::from_attributes(std::slice::from_ref(attr))?;
            Ok(Some(opts))
        }

        pub fn serialize_with(&self) -> Option<Expr> {
            self.serialize_with
                .as_ref()
                .map(|serialize_with| {
                    parse_quote! {
                        #serialize_with
                    }
                })
                .or_else(|| {
                    self.with.as_ref().map(|with| {
                        parse_quote! {
                            #with::serialize
                        }
                    })
                })
        }

        pub fn deserialize_with(&self) -> Option<Expr> {
            self.deserialize_with
                .as_ref()
                .map(|deserialize_with| {
                    parse_quote! {
                        #deserialize_with
                    }
                })
                .or_else(|| {
                    self.with.as_ref().map(|with| {
                        parse_quote! {
                            #with::deserialize
                        }
                    })
                })
        }
    }

    #[allow(clippy::large_enum_variant)]
    pub enum RootOpts {
        None,
        Value(RootValueOpts),
    }

    impl RootOpts {
        pub fn parse(ast: &syn::DeriveInput) -> Result<Self, DeriveError> {
            let value_opts = RootValueOpts::parse(ast)?;

            match value_opts {
                Some(value_opts) => Ok(RootOpts::Value(value_opts)),
                None => Ok(RootOpts::None),
            }
        }
    }
}

pub mod variants {
    use syn::Attribute;

    use crate::options::records::roots::{RootAttributeOpts, RootElementOpts};

    use super::*;

    #[derive(Default, FromAttributes, Clone)]
    #[darling(attributes(xvalue))]
    pub struct RootValueOpts {
        /// The text value to use for unit variants when serializing and deserializing.
        pub value: Option<String>,
        /// Set if whitespace should be ignored when deserializing.
        /// - `Any` (*default*): Ignore any whitespace.
        /// - `None`: Do not ignore whitespace.
        ///
        /// *Deserialize only*
        #[darling(default)]
        pub ignore_whitespace: IgnoreWhitespace,
        /// Set if comments should be ignored when deserializing.
        /// - `Any` (*default*): Ignore any comments.
        /// - `None`: Do not ignore comments.
        ///
        /// *Deserialize only*
        #[darling(default)]
        pub ignore_comments: IgnoreComments,
        /// Allow unknown values when deserializing.
        /// - `Any`: Allow any unknown values.
        /// - `AtEnd` (*default*): Allow unknown values only at the end of the element.
        /// - `None`: Do not allow unknown values at all.
        ///
        /// *Deserialize only*
        #[darling(default)]
        pub allow_unknown: AllowUnknown,
        /// Set if the order of values is important when serializing or deserializing.
        /// - `Strict`: The order of values must match the order in the struct or enum variant.
        /// - `None` (*default*): The order of values does not matter, but the values must be present.
        ///
        /// *Deserialize only*
        #[darling(default)]
        pub order: ElementOrder,
    }

    impl RootValueOpts {
        pub fn parse(attrs: &[Attribute]) -> Result<Option<Self>, DeriveError> {
            let Some(attr) = attrs.iter().find(|attr| attr.path().is_ident("xvalue")) else {
                return Ok(None);
            };

            let opts = Self::from_attributes(std::slice::from_ref(attr))?;
            Ok(Some(opts))
        }
    }

    pub enum DeserializeRootOpts {
        None,
        Element(RootElementOpts),
        Attribute(RootAttributeOpts),
        Value(RootValueOpts),
    }

    impl DeserializeRootOpts {
        pub fn parse(attrs: &[Attribute]) -> Result<Self, DeriveError> {
            let element_opts = RootElementOpts::parse(attrs)?;
            let attribute_opts = RootAttributeOpts::parse(attrs)?;
            let value_opts = RootValueOpts::parse(attrs)?;

            match (element_opts, attribute_opts, value_opts) {
                    (Some(element_opts), None, None) => Ok(Self::Element(element_opts)),
                    (None, Some(attribute_opts), None) => Ok(Self::Attribute(attribute_opts)),
                    (None, None, Some(value_opts)) => Ok(Self::Value(value_opts)),
                    (None, None, None) => Ok(Self::None),
                    _ => Err(DeriveError::custom("Wrong options. Only one of `xelement`, `xattribute`, or `xvalue` can be used for root elements.")),
                }
        }
    }
}
