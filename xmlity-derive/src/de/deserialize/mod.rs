mod attributes;
mod elements;
mod none;
mod values;
use attributes::StructAttributeVisitorBuilder;
use elements::StructElementVisitorBuilder;
use none::{EnumVisitorBuilder, SerializeNoneStructBuilder};
use quote::ToTokens;

use crate::{
    options::{
        XmlityRootAttributeDeriveOpts, XmlityRootElementDeriveOpts, XmlityRootValueDeriveOpts,
    },
    DeriveError, DeriveMacro,
};

use super::common::DeserializeBuilderExt;

enum DeriveDeserializeOption {
    None,
    Element(XmlityRootElementDeriveOpts),
    Attribute(XmlityRootAttributeDeriveOpts),
    Value(XmlityRootValueDeriveOpts),
}

impl DeriveDeserializeOption {
    pub fn parse(ast: &syn::DeriveInput) -> Result<Self, DeriveError> {
        let element_opts = XmlityRootElementDeriveOpts::parse(ast)?;
        let attribute_opts = XmlityRootAttributeDeriveOpts::parse(ast)?;
        let value_opts = XmlityRootValueDeriveOpts::parse(ast)?;

        match (element_opts, attribute_opts, value_opts) {
            (Some(element_opts), None, None) => Ok(DeriveDeserializeOption::Element(element_opts)),
            (None, Some(attribute_opts), None) => Ok(DeriveDeserializeOption::Attribute(attribute_opts)),
            (None, None, Some(value_opts)) => Ok(DeriveDeserializeOption::Value(value_opts)),
            (None, None, None) => Ok(DeriveDeserializeOption::None),
            _ => Err(DeriveError::custom("Wrong options. Only one of `xelement`, `xattribute`, or `xvalue` can be used for root elements.")),
        }
    }
}

pub struct DeriveDeserialize;

impl DeriveMacro for DeriveDeserialize {
    fn input_to_derive(ast: &syn::DeriveInput) -> Result<proc_macro2::TokenStream, DeriveError> {
        let opts = DeriveDeserializeOption::parse(ast)?;

        match (&ast.data, &opts) {
            // `xelement`
            (syn::Data::Struct(_), DeriveDeserializeOption::Element(opts)) => {
                StructElementVisitorBuilder::new(opts, ast)
                    .deserialize_trait_impl()
                    .map(|a| a.to_token_stream())
            }
            (syn::Data::Enum(_), DeriveDeserializeOption::Element(_)) => Err(DeriveError::custom(
                "`xelement` is not compatible with enums.",
            )),
            // `xattribute`
            (syn::Data::Struct(_), DeriveDeserializeOption::Attribute(opts)) => {
                StructAttributeVisitorBuilder::new(opts, ast)
                    .deserialize_trait_impl()
                    .map(|a| a.to_token_stream())
            }
            (syn::Data::Enum(_), DeriveDeserializeOption::Attribute(_)) => Err(
                DeriveError::custom("`xattribute` is not compatible with enums."),
            ),
            // `xvalue`
            (syn::Data::Enum(_), DeriveDeserializeOption::Value(opts)) => {
                EnumVisitorBuilder::new_with_value_opts(ast, opts)
                    .deserialize_trait_impl()
                    .map(|a| a.to_token_stream())
            }
            (syn::Data::Struct(_), DeriveDeserializeOption::Value(_)) => Err(DeriveError::custom(
                "`xvalue` is not compatible with structs.",
            )),
            // None
            (syn::Data::Struct(_), DeriveDeserializeOption::None) => {
                SerializeNoneStructBuilder::new(ast)
                    .deserialize_trait_impl()
                    .map(|a| a.to_token_stream())
            }
            (syn::Data::Enum(_), DeriveDeserializeOption::None) => EnumVisitorBuilder::new(ast)
                .deserialize_trait_impl()
                .map(|a| a.to_token_stream()),
            (syn::Data::Union(_), _) => Err(DeriveError::custom(
                "Unions are not supported for deserialization.",
            )),
        }
    }
}
