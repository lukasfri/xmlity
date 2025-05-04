mod attributes;
use std::borrow::Cow;

pub use attributes::SimpleDeserializeAttributeBuilder;
mod elements;
mod single_child_element;
pub use single_child_element::DeserializeSingleChildElementBuilder;
mod none;
mod variant;
use attributes::RecordDeserializeAttributeBuilder;
use elements::RecordDeserializeElementBuilder;
use none::{EnumVisitorBuilder, RecordDeserializeValueBuilder};
use quote::ToTokens;

use crate::{
    options::{enums, records},
    DeriveError, DeriveMacro,
};

use super::builders::{DeserializeBuilder, DeserializeBuilderExt};

use crate::common::{parse_enum_variant_derive_input, parse_struct_derive_input, RecordInput};

pub struct RecordDeserializeBuilder<'a, T: Fn(syn::Expr) -> syn::Expr> {
    pub input: &'a RecordInput<'a, T>,
    pub options: &'a records::roots::DeserializeRootOpts,
}

impl<'a, T: Fn(syn::Expr) -> syn::Expr> RecordDeserializeBuilder<'a, T> {
    pub fn new(
        input: &'a RecordInput<'a, T>,
        options: &'a records::roots::DeserializeRootOpts,
    ) -> Self {
        Self { input, options }
    }
}

impl<T: Fn(syn::Expr) -> syn::Expr> DeserializeBuilder for RecordDeserializeBuilder<'_, T> {
    fn deserialize_fn_body(
        &self,
        deserializer_ident: &syn::Ident,
        deserialize_lifetime: &syn::Lifetime,
    ) -> Result<Vec<syn::Stmt>, DeriveError> {
        use records::roots::DeserializeRootOpts;
        match &self.options {
            DeserializeRootOpts::Element(opts) => {
                RecordDeserializeElementBuilder::new(self.input, opts)
                    .deserialize_fn_body(deserializer_ident, deserialize_lifetime)
            }
            DeserializeRootOpts::Attribute(opts) => {
                RecordDeserializeAttributeBuilder::new(self.input, opts)
                    .to_builder()?
                    .deserialize_fn_body(deserializer_ident, deserialize_lifetime)
            }
            DeserializeRootOpts::Value(opts) => {
                RecordDeserializeValueBuilder::new(self.input, Some(opts))
                    .deserialize_fn_body(deserializer_ident, deserialize_lifetime)
            }
            DeserializeRootOpts::None => RecordDeserializeValueBuilder::new(self.input, None)
                .deserialize_fn_body(deserializer_ident, deserialize_lifetime),
        }
    }

    fn ident(&self) -> Cow<'_, syn::Ident> {
        Cow::Borrowed(self.input.impl_for_ident.as_ref())
    }

    fn generics(&self) -> Cow<'_, syn::Generics> {
        Cow::Borrowed(self.input.generics.as_ref())
    }
}

pub struct DeriveDeserialize;

impl DeriveMacro for DeriveDeserialize {
    fn input_to_derive(ast: &syn::DeriveInput) -> Result<proc_macro2::TokenStream, DeriveError> {
        match &ast.data {
            syn::Data::Struct(_) => {
                let opts = records::roots::DeserializeRootOpts::parse(&ast.attrs)?;

                let record = parse_struct_derive_input(ast)?;
                RecordDeserializeBuilder::new(&record, &opts)
                    .deserialize_trait_impl()
                    .map(|a| a.to_token_stream())
            }
            syn::Data::Enum(_) => {
                let opts = enums::roots::RootOpts::parse(ast)?;

                let value_opts = match &opts {
                    enums::roots::RootOpts::None => None,
                    enums::roots::RootOpts::Value(opts) => Some(opts),
                };

                EnumVisitorBuilder::new(ast, value_opts)
                    .deserialize_trait_impl()
                    .map(|a| a.to_token_stream())
            }
            syn::Data::Union(_) => Err(DeriveError::custom(
                "Unions are not supported for deserialization.",
            )),
        }
    }
}
