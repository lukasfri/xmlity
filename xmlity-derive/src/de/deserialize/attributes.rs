use std::{borrow::Cow, ops::Deref};

use proc_macro2::Span;
use syn::{parse_quote, Ident, Lifetime, LifetimeParam, Stmt, Type};

use crate::{
    common::{non_bound_generics, ExpandedName, StructTypeWithFields},
    de::builders::{DeserializeBuilder, VisitorBuilder, VisitorBuilderExt},
    options::{
        records::{
            fields::{FieldOpts, ValueOpts},
            roots::RootAttributeOpts,
        },
        FieldWithOpts, WithExpandedNameExt,
    },
    DeriveError,
};

use super::RecordInput;

pub struct RecordDeserializeAttributeBuilder<'a, T: Fn(syn::Expr) -> syn::Expr> {
    opts: &'a RootAttributeOpts,
    ast: &'a RecordInput<'a, T>,
}

impl<'a, T: Fn(syn::Expr) -> syn::Expr> RecordDeserializeAttributeBuilder<'a, T> {
    pub fn new(ast: &'a RecordInput<'a, T>, opts: &'a RootAttributeOpts) -> Self {
        Self { opts, ast }
    }

    pub fn to_builder(&self) -> Result<StructDeserializeAttributeBuilder, DeriveError> {
        let RecordInput {
            impl_for_ident: ident,
            generics,
            fields,
            ..
        } = &self.ast;
        let RootAttributeOpts {
            deserialize_any_name,
            ..
        } = self.opts;

        let required_expanded_name = if *deserialize_any_name {
            None
        } else {
            Some(
                self.opts
                    .expanded_name(ident.to_string().as_str())
                    .into_owned(),
            )
        };

        let struct_type = match &fields {
            StructTypeWithFields::Named(fields_named) if fields_named.len() != 1 => {
                return Err(DeriveError::custom(format!(
                    "Expected a single field for attribute deserialization, found {}",
                    fields_named.len()
                )))
            }
            StructTypeWithFields::Named(fields_named) => {
                let field = &fields_named[0];
                StructTypeWithFields::Named(field.clone())
            }
            StructTypeWithFields::Unnamed(fields_unnamed) if fields_unnamed.len() != 1 => {
                return Err(DeriveError::custom(format!(
                    "Expected a single field for attribute deserialization, found {}",
                    fields_unnamed.len()
                )))
            }
            StructTypeWithFields::Unnamed(fields_unnamed) => {
                let field = &fields_unnamed[0];
                StructTypeWithFields::Unnamed(field.clone())
            }
            StructTypeWithFields::Unit => StructTypeWithFields::Unit,
        };

        Ok(StructDeserializeAttributeBuilder {
            ident,
            generics,
            required_expanded_name,
            struct_type,
        })
    }
}

pub struct SimpleDeserializeAttributeBuilder<'a> {
    pub ident: &'a syn::Ident,
    pub generics: &'a syn::Generics,
    pub required_expanded_name: Option<ExpandedName<'static>>,
    pub item_type: &'a syn::Type,
}

impl SimpleDeserializeAttributeBuilder<'_> {
    fn value_access_ident(&self) -> Ident {
        Ident::new("__value", Span::call_site())
    }

    pub fn struct_definition(&self) -> syn::ItemStruct {
        let Self {
            ident, item_type, ..
        } = self;

        let value_access_ident = self.value_access_ident();

        parse_quote! {
            struct #ident {
                #value_access_ident: #item_type,
            }
        }
    }

    pub fn to_builder(&self) -> StructDeserializeAttributeBuilder {
        StructDeserializeAttributeBuilder {
            ident: self.ident,
            generics: self.generics,
            required_expanded_name: self.required_expanded_name.clone(),
            struct_type: StructTypeWithFields::Named(FieldWithOpts {
                field_ident: self.value_access_ident(),
                field_type: self.item_type.clone(),
                options: FieldOpts::Value(crate::options::records::fields::ChildOpts::Value(
                    ValueOpts::default(),
                )),
            }),
        }
    }
}

pub struct StructDeserializeAttributeBuilder<'a> {
    pub ident: &'a syn::Ident,
    pub generics: &'a syn::Generics,
    pub required_expanded_name: Option<ExpandedName<'static>>,
    pub struct_type: StructTypeWithFields<
        FieldWithOpts<syn::Ident, FieldOpts>,
        FieldWithOpts<syn::Index, FieldOpts>,
    >,
}

impl VisitorBuilder for StructDeserializeAttributeBuilder<'_> {
    fn visit_attribute_fn_body(
        &self,
        visitor_lifetime: &Lifetime,
        attribute_access_ident: &Ident,
        access_type: &Type,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let Self {
            ident,
            required_expanded_name,
            struct_type,
            ..
        } = self;

        let xml_name_identification = required_expanded_name.as_ref().map::<Stmt, _>(|qname| {
              parse_quote! {
                  ::xmlity::de::AttributeAccessExt::ensure_name::<<#access_type as ::xmlity::de::AttributeAccess<#visitor_lifetime>>::Error>(&#attribute_access_ident, &#qname)?;
              }
          });

        let deserialization_impl: Vec<Stmt> = match &struct_type {
            StructTypeWithFields::Named(FieldWithOpts {
                field_ident,
                field_type,
                ..
            }) => {
                parse_quote! {
                    <#field_type as ::core::str::FromStr>::from_str(::xmlity::de::AttributeAccess::value(&#attribute_access_ident))
                        .map(|a| #ident {#field_ident: a})
                        .map_err(::xmlity::de::Error::custom)
                }
            }
            StructTypeWithFields::Unnamed(FieldWithOpts { field_type, .. }) => {
                parse_quote! {
                    <#field_type as ::core::str::FromStr>::from_str(::xmlity::de::AttributeAccess::value(&#attribute_access_ident))
                        .map(#ident)
                        .map_err(::xmlity::de::Error::custom)
                }
            }
            StructTypeWithFields::Unit => {
                parse_quote! {
                    Ok(#ident)
                }
            }
        };

        Ok(Some(parse_quote! {
            #xml_name_identification

            #(#deserialization_impl)*
        }))
    }

    fn visitor_definition(&self) -> Result<syn::ItemStruct, DeriveError> {
        let ident = self.visitor_ident();
        let generics = self.visitor_generics();

        let non_bound_generics = non_bound_generics(generics.deref());

        let mut deserialize_generics = generics.into_owned();

        let visitor_ident = Ident::new("__Visitor", Span::mixed_site());
        let visitor_lifetime = Lifetime::new("'__visitor", Span::mixed_site());

        deserialize_generics.params.insert(
            0,
            syn::GenericParam::Lifetime(LifetimeParam::new(visitor_lifetime.clone())),
        );

        Ok(parse_quote! {
            struct #visitor_ident #deserialize_generics {
                marker: ::core::marker::PhantomData<#ident #non_bound_generics>,
                lifetime: ::core::marker::PhantomData<&#visitor_lifetime ()>,
            }
        })
    }

    fn visitor_ident(&self) -> Cow<'_, Ident> {
        Cow::Borrowed(self.ident)
    }

    fn visitor_generics(&self) -> Cow<'_, syn::Generics> {
        Cow::Borrowed(self.generics)
    }
}

impl DeserializeBuilder for StructDeserializeAttributeBuilder<'_> {
    fn deserialize_fn_body(
        &self,
        deserializer_ident: &Ident,
        _deserialize_lifetime: &Lifetime,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let formatter_expecting = format!("struct {}", self.visitor_ident());

        let visitor_ident = Ident::new("__Visitor", Span::mixed_site());

        let visitor_def = self.visitor_definition()?;
        let visitor_trait_impl = self.visitor_trait_impl(&visitor_ident, &formatter_expecting)?;

        Ok(parse_quote! {
            #visitor_def

            #visitor_trait_impl

            ::xmlity::de::Deserializer::deserialize_any(#deserializer_ident, #visitor_ident {
                lifetime: ::core::marker::PhantomData,
                marker: ::core::marker::PhantomData,
            })
        })
    }

    fn ident(&self) -> Cow<'_, Ident> {
        Cow::Borrowed(self.ident)
    }

    fn generics(&self) -> Cow<'_, syn::Generics> {
        Cow::Borrowed(self.generics)
    }
}
