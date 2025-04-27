mod attributes;
mod elements;
mod none;
mod values;
use attributes::StructAttributeVisitorBuilder;
use elements::StructElementVisitorBuilder;
use none::{EnumVisitorBuilder, SerializeNoneStructBuilder};
use quote::ToTokens;

use crate::{
    options::{enums, structs},
    DeriveError, DeriveMacro,
};

use super::common::DeserializeBuilderExt;

pub struct DeriveDeserialize;

impl DeriveMacro for DeriveDeserialize {
    fn input_to_derive(ast: &syn::DeriveInput) -> Result<proc_macro2::TokenStream, DeriveError> {
        match &ast.data {
            // `xelement`
            syn::Data::Struct(_) => {
                let opts = structs::roots::DeserializeRootOpts::parse(ast)?;

                match opts {
                    structs::roots::DeserializeRootOpts::None => {
                        SerializeNoneStructBuilder::new(ast)
                            .deserialize_trait_impl()
                            .map(|a| a.to_token_stream())
                    }
                    structs::roots::DeserializeRootOpts::Element(opts) => {
                        StructElementVisitorBuilder::new(&opts, ast)
                            .deserialize_trait_impl()
                            .map(|a| a.to_token_stream())
                    }
                    structs::roots::DeserializeRootOpts::Attribute(opts) => {
                        StructAttributeVisitorBuilder::new(&opts, ast)
                            .deserialize_trait_impl()
                            .map(|a| a.to_token_stream())
                    }
                    structs::roots::DeserializeRootOpts::Value(_opts) => Err(DeriveError::custom(
                        "`xvalue` is not compatible with structs.",
                    )),
                }
            }
            syn::Data::Enum(_) => {
                let opts = enums::roots::RootOpts::parse(ast)?;

                match opts {
                    enums::roots::RootOpts::None => EnumVisitorBuilder::new(ast)
                        .deserialize_trait_impl()
                        .map(|a| a.to_token_stream()),
                    enums::roots::RootOpts::Value(opts) => {
                        EnumVisitorBuilder::new_with_value_opts(ast, &opts)
                            .deserialize_trait_impl()
                            .map(|a| a.to_token_stream())
                    }
                }
            }
            syn::Data::Union(_) => Err(DeriveError::custom(
                "Unions are not supported for deserialization.",
            )),
        }
    }
}
