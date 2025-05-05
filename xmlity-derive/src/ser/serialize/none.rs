use std::borrow::Cow;

use proc_macro2::Span;
use quote::quote;
use syn::{parse_quote, Arm, Data, DataEnum, DeriveInput, Fields, Generics, Ident, Stmt};

use crate::{
    common::{self, value_deconstructor, FieldIdent, RecordInput},
    options::{
        enums::roots::RootValueOpts as EnumRootVolueOpts,
        records::{
            self,
            roots::{RootValueOpts as RecordRootValueOpts, SerializeRootOpts},
        },
    },
    ser::{
        builders::{SerializeBuilder, SerializeBuilderExt},
        common::{element_fields, seq_field_serializer},
    },
    DeriveError,
};

use super::variant::SerializeVariantBuilder;

pub struct RecordSerializeValueBuilder<'a, T: Fn(syn::Expr) -> syn::Expr> {
    input: &'a RecordInput<'a, T>,
    options: Option<&'a RecordRootValueOpts>,
}

impl<'a, T: Fn(syn::Expr) -> syn::Expr> RecordSerializeValueBuilder<'a, T> {
    pub fn new(ast: &'a RecordInput<'a, T>, options: Option<&'a RecordRootValueOpts>) -> Self {
        Self {
            input: ast,
            options,
        }
    }
}

impl<T: Fn(syn::Expr) -> syn::Expr> SerializeBuilder for RecordSerializeValueBuilder<'_, T> {
    fn serialize_fn_body(
        &self,
        serializer_access: &Ident,
        _serializer_type: &syn::Type,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let seq_access_ident = Ident::new("__seq_access", proc_macro2::Span::call_site());

        let fields: Vec<_> = match (&self.input.fields, &self.options) {
            (common::StructTypeWithFields::Named(fields), _) => fields
                .iter()
                .cloned()
                .map(|f| f.map_ident(FieldIdent::Named))
                .collect(),
            (common::StructTypeWithFields::Unnamed(fields), _) => fields
                .iter()
                .cloned()
                .map(|f| f.map_ident(FieldIdent::Indexed))
                .collect(),
            (
                common::StructTypeWithFields::Unit,
                Some(RecordRootValueOpts {
                    value: Some(value), ..
                }),
            ) => {
                return Ok(parse_quote! {
                    ::xmlity::Serializer::serialize_text(#serializer_access, #value)
                });
            }
            (common::StructTypeWithFields::Unit, _) => {
                return Ok(parse_quote! {
                    ::xmlity::Serializer::serialize_none(#serializer_access)
                });
            }
        };

        let record_path = self.input.record_path.as_ref();

        let value_deconstructor = value_deconstructor(
            self.input.constructor_path.as_ref(),
            &parse_quote!(&#record_path),
            &self.input.fields,
            self.input.fallable_deconstruction,
        );

        let value_fields =
            seq_field_serializer(quote! {#seq_access_ident}, element_fields(fields)?)?;

        Ok(parse_quote! {
            #(#value_deconstructor)*
            let mut #seq_access_ident = ::xmlity::Serializer::serialize_seq(#serializer_access)?;
            #value_fields
            ::xmlity::ser::SerializeSeq::end(#seq_access_ident)
        })
    }

    fn ident(&self) -> Cow<'_, Ident> {
        Cow::Borrowed(self.input.impl_for_ident.as_ref())
    }

    fn generics(&self) -> Cow<'_, Generics> {
        Cow::Borrowed(self.input.generics.as_ref())
    }
}

pub struct DeriveEnum<'a> {
    ast: &'a syn::DeriveInput,
    value_opts: Option<&'a EnumRootVolueOpts>,
}

impl<'a> DeriveEnum<'a> {
    pub fn new(ast: &'a syn::DeriveInput, value_opts: Option<&'a EnumRootVolueOpts>) -> Self {
        Self { ast, value_opts }
    }
}

impl SerializeBuilder for DeriveEnum<'_> {
    fn serialize_fn_body(
        &self,
        serializer_access: &Ident,
        _serializer_type: &syn::Type,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let DeriveInput {
            ident: enum_ident,
            data,
            generics: enum_generics,
            ..
        } = self.ast;

        let Data::Enum(DataEnum { variants, .. }) = &data else {
            unreachable!()
        };

        let variants = variants
            .iter()
            .map::<Result<Arm, DeriveError>, _>(|variant| {
                let variant_ident = &variant.ident;

                let record = common::parse_enum_variant_derive_input(
                    enum_ident,
                    enum_generics,
                    variant,
                    variants.len() > 1,
                )?;

                let mut variant_opts = records::roots::SerializeRootOpts::parse(&variant.attrs)?;
                if let SerializeRootOpts::Value(records::roots::RootValueOpts {
                    value: value @ None,
                }) = &mut variant_opts
                {
                    let ident_value = self
                        .value_opts
                        .as_ref()
                        .map(|a| a.rename_all)
                        .unwrap_or_default()
                        .apply_to_variant(&variant.ident.to_string());

                    *value = Some(ident_value);
                }
                if let SerializeRootOpts::None = variant_opts {
                    let ident_value = self
                        .value_opts
                        .as_ref()
                        .map(|a| a.rename_all)
                        .unwrap_or_default()
                        .apply_to_variant(&variant.ident.to_string());

                    variant_opts = SerializeRootOpts::Value(records::roots::RootValueOpts {
                        value: Some(ident_value),
                    });
                }

                let sub_value_ident = Ident::new("__sub_value", Span::call_site());

                let matcher = match &variant.fields {
                    Fields::Unit => quote! { #sub_value_ident @ #enum_ident::#variant_ident },
                    Fields::Unnamed(fields) => {
                        let fields = fields.unnamed.iter().map(|_| quote! { _ });
                        quote! { #sub_value_ident @ #enum_ident::#variant_ident(#(#fields),*) }
                    }
                    Fields::Named(_) => {
                        quote! { #sub_value_ident @ #enum_ident::#variant_ident { .. } }
                    }
                };

                let variant_builder = SerializeVariantBuilder::new(&record, &variant_opts);

                let definition = variant_builder.definition();
                let serialize_impl = variant_builder.serialize_trait_impl()?;

                let serialize_expr = variant_builder.serialize_expr(&sub_value_ident);

                Ok(parse_quote!(
                    __inner @ #matcher => {
                        #definition
                        #serialize_impl

                        ::xmlity::Serialize::serialize(&#serialize_expr, #serializer_access)
                    }
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(parse_quote! {
            match self {
                #(#variants)*
            }
        })
    }

    fn ident(&self) -> Cow<'_, Ident> {
        Cow::Borrowed(&self.ast.ident)
    }

    fn generics(&self) -> Cow<'_, Generics> {
        Cow::Borrowed(&self.ast.generics)
    }
}
