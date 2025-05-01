mod none;

pub use none::{DeriveEnum, DeriveNoneStruct};
mod element;
pub use element::{DeriveElementStruct, SingleChildSerializeElementBuilder};

use quote::ToTokens;
use syn::DeriveInput;

use crate::options::{enums, structs};
use crate::{DeriveError, DeriveMacro};

use super::builders::SerializeBuilderExt;

pub struct DeriveSerialize;

impl DeriveMacro for DeriveSerialize {
    fn input_to_derive(ast: &DeriveInput) -> Result<proc_macro2::TokenStream, DeriveError> {
        match &ast.data {
            syn::Data::Struct(_) => {
                let opts = structs::roots::SerializeRootOpts::parse(ast)?;
                match opts {
                    structs::roots::SerializeRootOpts::Element(opts) => {
                        DeriveElementStruct::new(&opts, ast)
                            .to_builder()?
                            .serialize_trait_impl()
                            .map(|a| a.to_token_stream())
                    }
                    structs::roots::SerializeRootOpts::None => DeriveNoneStruct::new(ast)
                        .serialize_trait_impl()
                        .map(|a| a.to_token_stream()),
                    structs::roots::SerializeRootOpts::Value(_) => Err(DeriveError::custom(
                        "`xvalue` is not compatible with structs.",
                    )),
                }
            }
            syn::Data::Enum(_) => {
                let opts = enums::roots::RootOpts::parse(ast)?;

                match opts {
                    enums::roots::RootOpts::Value(opts) => DeriveEnum::new(ast, Some(&opts))
                        .serialize_trait_impl()
                        .map(|a| a.to_token_stream()),
                    enums::roots::RootOpts::None => DeriveEnum::new(ast, None)
                        .serialize_trait_impl()
                        .map(|a| a.to_token_stream()),
                }
            }
            syn::Data::Union(_) => Err(DeriveError::custom(
                "Unions are not supported for serialization.",
            )),
        }
    }
}
