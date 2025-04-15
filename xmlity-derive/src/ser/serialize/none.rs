use quote::quote;
use syn::{parse_quote, Arm, Data, DataEnum, DataStruct, DeriveInput, Ident, Stmt};

use crate::DeriveError;

use super::SerializeBuilder;

pub struct DeriveNoneStruct {}

impl DeriveNoneStruct {
    pub fn new() -> Self {
        Self {}
    }
}

impl SerializeBuilder for DeriveNoneStruct {
    fn serialize_fn_body(
        &self,
        ast: &syn::DeriveInput,
        serializer_access: &Ident,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let seq_access_ident = Ident::new("__seq_access", proc_macro2::Span::call_site());

        let Data::Struct(DataStruct { fields, .. }) = &ast.data else {
            unreachable!()
        };

        match fields {
            syn::Fields::Named(_) | syn::Fields::Unnamed(_) => {
                let value_fields = crate::ser::seq_field_serializer(
                    quote! {#seq_access_ident},
                    crate::ser::element_fields(ast)?,
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
}

pub struct DeriveNoneEnum {}

impl DeriveNoneEnum {
    pub fn new() -> Self {
        Self {}
    }
}

impl SerializeBuilder for DeriveNoneEnum {
    fn serialize_fn_body(
        &self,
        ast: &syn::DeriveInput,
        serializer_access: &Ident,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let DeriveInput { ident, data, .. } = ast;

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
                    syn::Fields::Unit => Err(DeriveError::custom(
                        "Unsupported unit variant in non-value enum.",
                    )),
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(parse_quote! {
            match self {
                #(#variants)*
            }
        })
    }
}
