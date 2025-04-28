use std::borrow::Cow;

use quote::quote;
use syn::{parse_quote, Arm, Data, DataEnum, DataStruct, DeriveInput, Generics, Ident, Stmt};

use crate::{
    options::enums::{
        roots::RootValueOpts,
        variants::{ValueOpts, VariantOpts},
    },
    DeriveError,
};

use super::SerializeBuilder;

pub struct DeriveNoneStruct<'a> {
    ast: &'a DeriveInput,
}

impl<'a> DeriveNoneStruct<'a> {
    pub fn new(ast: &'a DeriveInput) -> Self {
        Self { ast }
    }
}

impl SerializeBuilder for DeriveNoneStruct<'_> {
    fn serialize_fn_body(
        &self,
        serializer_access: &Ident,
        _serializer_type: &syn::Type,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let seq_access_ident = Ident::new("__seq_access", proc_macro2::Span::call_site());

        let Data::Struct(DataStruct { fields, .. }) = &self.ast.data else {
            unreachable!()
        };

        match fields {
            syn::Fields::Named(_) | syn::Fields::Unnamed(_) => {
                let value_fields = crate::ser::seq_field_serializer(
                    quote! {#seq_access_ident},
                    crate::ser::element_fields(self.ast)?,
                );

                Ok(parse_quote! {
                    let mut #seq_access_ident = ::xmlity::Serializer::serialize_seq(#serializer_access)?;
                    #value_fields
                    ::xmlity::ser::SerializeSeq::end(#seq_access_ident)
                })
            }
            syn::Fields::Unit => Ok(parse_quote! {
                ::xmlity::Serializer::serialize_none(serializer)?;
            }),
        }
    }

    fn ident(&self) -> Cow<'_, Ident> {
        Cow::Borrowed(&self.ast.ident)
    }

    fn generics(&self) -> Cow<'_, Generics> {
        Cow::Borrowed(&self.ast.generics)
    }
}

pub struct DeriveEnum<'a> {
    ast: &'a syn::DeriveInput,
    value_opts: Option<&'a RootValueOpts>,
}

impl<'a> DeriveEnum<'a> {
    pub fn new(ast: &'a syn::DeriveInput, value_opts: Option<&'a RootValueOpts>) -> Self {
        Self { ast, value_opts }
    }
}

impl SerializeBuilder for DeriveEnum<'_> {
    fn serialize_fn_body(
        &self,
        serializer_access: &Ident,
        _serializer_type: &syn::Type,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let DeriveInput { ident, data, .. } = self.ast;

        let Data::Enum(DataEnum { variants, .. }) = &data else {
            unreachable!()
        };

        let variants = variants
            .iter()
            .map::<Result<Arm, DeriveError>, _>(|variant| {
                let variant_ident = &variant.ident;

                match &variant.fields {
                    syn::Fields::Named(_fields) => {
                        Err(DeriveError::custom("Named fields are not supported yet"))
                    }
                    syn::Fields::Unnamed(fields) if fields.unnamed.is_empty() => Ok(parse_quote! {
                        #ident::#variant_ident() => {
                            ::xmlity::Serialize::serialize(&__v, #serializer_access)
                        },
                    }),
                    syn::Fields::Unnamed(fields) if fields.unnamed.len() != 1 => {
                        Err(DeriveError::custom(
                            "Enum variants with more than one field are not supported",
                        ))
                    }
                    syn::Fields::Unnamed(_) => Ok(parse_quote! {
                        #ident::#variant_ident(__v) => {
                            ::xmlity::Serialize::serialize(&__v, #serializer_access)
                        },
                    }),
                    syn::Fields::Unit => {
                        let variant_ident = &variant.ident;

                        let variant_opts = VariantOpts::from_variant(variant)?;

                        let value = variant_opts
                            .as_ref()
                            .and_then(|a| match a {
                                VariantOpts::Value(ValueOpts {
                                    value: Some(value), ..
                                }) => Some(value.clone()),
                                _ => None,
                            })
                            .unwrap_or_else(|| {
                                self.value_opts
                                    .as_ref()
                                    .map(|a| a.rename_all)
                                    .unwrap_or_default()
                                    .apply_to_variant(&variant_ident.to_string())
                            });

                        Ok(parse_quote! {
                            #ident::#variant_ident => {
                                ::xmlity::Serialize::serialize(&#value, #serializer_access)
                            },
                        })
                    }
                }
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
