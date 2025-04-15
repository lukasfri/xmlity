mod attributes;
mod elements;
mod none;
mod values;
use attributes::StructAttributeVisitorBuilder;
use elements::StructElementVisitorBuilder;
use none::{EnumNoneVisitorBuilder, SerializeNoneStructBuilder};
use quote::ToTokens;
use values::EnumValueVisitorBuilder;

use crate::{DeriveDeserializeOption, DeriveError, DeriveMacro};

use super::common::DeserializeBuilderExt;

pub struct DeriveDeserialize;

impl DeriveMacro for DeriveDeserialize {
    fn input_to_derive(ast: &syn::DeriveInput) -> Result<proc_macro2::TokenStream, DeriveError> {
        let opts = DeriveDeserializeOption::parse(ast)?;

        match (&ast.data, &opts) {
            (syn::Data::Struct(_), DeriveDeserializeOption::Element(opts)) => {
                StructElementVisitorBuilder::new(opts)
                    .deserialize_trait_impl(ast)
                    .map(|a| a.to_token_stream())
            }
            (syn::Data::Struct(_), DeriveDeserializeOption::Attribute(opts)) => {
                StructAttributeVisitorBuilder::new(opts)
                    .deserialize_trait_impl(ast)
                    .map(|a| a.to_token_stream())
            }
            (syn::Data::Struct(_), DeriveDeserializeOption::None) => {
                SerializeNoneStructBuilder::new()
                    .deserialize_trait_impl(ast)
                    .map(|a| a.to_token_stream())
            }
            (syn::Data::Struct(_), DeriveDeserializeOption::Value(_opts)) => Err(
                DeriveError::custom("Structs with value options are not supported yet"),
            ),
            (syn::Data::Enum(_), DeriveDeserializeOption::None) => EnumNoneVisitorBuilder::new()
                .deserialize_trait_impl(ast)
                .map(|a| a.to_token_stream()),
            (syn::Data::Enum(_), DeriveDeserializeOption::Value(value_opts)) => {
                EnumValueVisitorBuilder::new(value_opts)
                    .deserialize_trait_impl(ast)
                    .map(|a| a.to_token_stream())
            }
            (syn::Data::Union(_), _) => Err(DeriveError::custom("Unions are not supported yet")),
            _ => Err(DeriveError::custom(
                "Wrong options. Unsupported deserialize.",
            )),
        }
    }
}
