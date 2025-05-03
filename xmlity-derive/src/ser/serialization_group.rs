use std::borrow::Cow;

use quote::{quote, ToTokens};
use syn::{parse_quote, DataStruct, DeriveInput, Generics, Ident, Stmt};

use crate::{options::records::roots::RootGroupOpts, DeriveError, DeriveMacro};

use super::builders::{SerializationGroupBuilder, SerializationGroupBuilderExt};

#[allow(unused)]
pub struct DeriveSerializationGroupStruct<'a> {
    ast: &'a syn::DeriveInput,
    opts: &'a RootGroupOpts,
}

impl<'a> DeriveSerializationGroupStruct<'a> {
    fn new(ast: &'a syn::DeriveInput, opts: &'a RootGroupOpts) -> Self {
        Self { ast, opts }
    }
}

impl SerializationGroupBuilder for DeriveSerializationGroupStruct<'_> {
    fn serialize_attributes_fn_body(
        &self,
        element_access_ident: &Ident,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let serialize_attributes_implementation = super::attribute_group_field_serializer(
            quote! { #element_access_ident},
            crate::ser::attribute_group_fields(crate::ser::fields(self.ast)?)?,
        )?;

        Ok(parse_quote! {
            #serialize_attributes_implementation
            ::core::result::Result::Ok(())
        })
    }

    fn serialize_children_fn_body(
        &self,
        children_access_ident: &Ident,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let serialize_children_implementation = super::element_group_field_serializer(
            quote! { #children_access_ident},
            crate::ser::element_group_fields(crate::ser::fields(self.ast)?)?,
        )?;

        Ok(parse_quote! {
            #serialize_children_implementation
            ::core::result::Result::Ok(())
        })
    }

    fn ident(&self) -> Cow<'_, Ident> {
        Cow::Borrowed(&self.ast.ident)
    }

    fn generics(&self) -> Cow<'_, Generics> {
        Cow::Borrowed(&self.ast.generics)
    }
}

enum SerializationGroupOption {
    Group(RootGroupOpts),
}

impl SerializationGroupOption {
    pub fn parse(ast: &DeriveInput) -> Result<Self, DeriveError> {
        let group_opts = RootGroupOpts::parse(&ast.attrs)?.unwrap_or_default();

        Ok(SerializationGroupOption::Group(group_opts))
    }
}

pub struct DeriveSerializationGroup;

impl DeriveMacro for DeriveSerializationGroup {
    fn input_to_derive(ast: &DeriveInput) -> Result<proc_macro2::TokenStream, DeriveError> {
        let SerializationGroupOption::Group(opts) = SerializationGroupOption::parse(ast)?;

        match &ast.data {
            syn::Data::Struct(DataStruct {
                fields: syn::Fields::Unit,
                ..
            }) => Err(DeriveError::custom(
                "Unit structs are not supported for serialization groups.",
            )),
            syn::Data::Struct(_) => DeriveSerializationGroupStruct::new(ast, &opts)
                .serialization_group_trait_impl()
                .map(|a| a.to_token_stream()),
            syn::Data::Enum(_) => Err(DeriveError::custom(
                "Enums are not supported for serialization groups.",
            )),
            syn::Data::Union(_) => Err(DeriveError::custom(
                "Unions are not supported for serialization groups.",
            )),
        }
    }
}
