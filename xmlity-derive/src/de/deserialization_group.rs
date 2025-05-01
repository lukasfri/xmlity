use std::{borrow::Cow, iter};

use proc_macro2::Span;
use quote::quote;
use syn::{parse_quote, DeriveInput, Ident, Index, ItemStruct, Lifetime, LifetimeParam, Stmt};

use crate::{
    common::{FieldIdent, StructType},
    options::{
        structs::{
            fields::{AttributeOpts, ChildOpts, GroupOpts},
            roots::RootGroupOpts,
        },
        FieldWithOpts, GroupOrder,
    },
    DeriveError, DeriveMacro, DeriveResult,
};

use super::{
    builders::{DeserializationGroupBuilderBuilder, DeserializationGroupBuilderContentExt},
    common::{
        all_attributes_done_expr, attribute_fields, attribute_group_fields,
        builder_attribute_field_visitor, builder_element_field_visitor, element_fields,
        element_group_fields, group_fields,
    },
};

use super::common::{all_elements_done_expr, constructor_expr, struct_definition_expr};

pub struct DeriveDeserializationGroupStruct<'a> {
    opts: &'a RootGroupOpts,
    ast: &'a DeriveInput,
}

impl<'a> DeriveDeserializationGroupStruct<'a> {
    pub fn new(ast: &'a DeriveInput, opts: &'a RootGroupOpts) -> Self {
        Self { ast, opts }
    }

    pub fn constructor_type(ast: &syn::DeriveInput) -> StructType {
        let data_struct = match ast.data {
            syn::Data::Struct(ref data_struct) => data_struct,
            _ => unreachable!(),
        };
        match &data_struct.fields {
            syn::Fields::Named(_) => StructType::Named,
            syn::Fields::Unnamed(_) => StructType::Unnamed,
            _ => unreachable!(),
        }
    }
}

impl DeserializationGroupBuilderBuilder for DeriveDeserializationGroupStruct<'_> {
    fn contribute_attributes_fn_body(
        &self,
        attributes_access_ident: &Ident,
        _deserialize_lifetime: &Lifetime,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let attribute_visit = builder_attribute_field_visitor(
            attributes_access_ident,
            quote! {self.},
            attribute_group_fields(self.ast)?,
            parse_quote! {return ::core::result::Result::Ok(false);},
            parse_quote! {return ::core::result::Result::Ok(true);},
            parse_quote! {return ::core::result::Result::Ok(true);},
            match self.opts.attribute_order {
                GroupOrder::Strict => parse_quote! {},
                GroupOrder::Loose => parse_quote! {return ::core::result::Result::Ok(false);},
                GroupOrder::None => parse_quote! {},
            },
            false,
        )?;

        Ok(Some(parse_quote! {
                #(#attribute_visit)*

                Ok(false)

        }))
    }

    fn attributes_done_fn_body(
        &self,
        _deserialize_lifetime: &Lifetime,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let expr = all_attributes_done_expr(attribute_group_fields(self.ast)?, quote! {self.});

        Ok(Some(parse_quote!(
            #expr
        )))
    }

    fn contribute_elements_fn_body(
        &self,
        elements_access_ident: &Ident,
        _deserialize_lifetime: &Lifetime,
    ) -> DeriveResult<Option<Vec<Stmt>>> {
        let element_visit = builder_element_field_visitor(
            elements_access_ident,
            quote! {self.},
            element_group_fields(self.ast)?,
            parse_quote! {return ::core::result::Result::Ok(false);},
            parse_quote! {return ::core::result::Result::Ok(true);},
            parse_quote! {return ::core::result::Result::Ok(true);},
            match self.opts.children_order {
                GroupOrder::Strict => parse_quote! {},
                GroupOrder::Loose => parse_quote! {return ::core::result::Result::Ok(false);},
                GroupOrder::None => parse_quote! {},
            },
            match self.opts.children_order {
                GroupOrder::Strict => true,
                GroupOrder::Loose | GroupOrder::None => false,
            },
        )?;

        Ok(Some(parse_quote! {
            #(#element_visit)*

            ::core::result::Result::Ok(false)
        }))
    }

    fn elements_done_fn_body(
        &self,
        _deserialize_lifetime: &Lifetime,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let expr = all_elements_done_expr(element_group_fields(self.ast)?, quote! {self.});

        Ok(Some(parse_quote!(
            #expr
        )))
    }

    fn finish_fn_body(&self) -> Result<Vec<Stmt>, DeriveError> {
        let finish_constructor = finish_constructor_expr(
            quote! {Self::Value},
            element_fields(self.ast)?,
            attribute_fields(self.ast)?,
            group_fields(self.ast)?,
            &Self::constructor_type(self.ast),
        );

        Ok(parse_quote! {
          ::std::result::Result::Ok(#finish_constructor)
        })
    }

    fn builder_definition(
        &self,
        builder_ident: &Ident,
        deserialize_lifetime: &Lifetime,
    ) -> Result<ItemStruct, DeriveError> {
        let local_value_expressions_constructors = attribute_fields(self.ast)?
            .into_iter()
            .map(
                |FieldWithOpts {
                     field_ident,
                     field_type,
                     ..
                 }| {
                    let expression = parse_quote! {
                        ::core::option::Option<#field_type>
                    };
                    (field_ident, expression)
                },
            )
            .chain(element_fields(self.ast)?.into_iter().map(
                |FieldWithOpts {
                     field_ident,
                     field_type,
                     ..
                 }| {
                    let expression = parse_quote! {
                        ::core::option::Option<#field_type>
                    };
                    (field_ident, expression)
                },
            ));
        let group_value_expressions_constructors = group_fields(self.ast)?.into_iter().map(
            |FieldWithOpts {
                field_ident,
                 field_type,
                 ..
             }| {
                let expression = parse_quote! {
                    <#field_type as ::xmlity::de::DeserializationGroup<#deserialize_lifetime>>::Builder
                };

                (field_ident, expression)
            },
        );

        let value_expressions_constructors = local_value_expressions_constructors
            .chain(group_value_expressions_constructors)
            .chain(iter::once((
                match Self::constructor_type(self.ast) {
                    StructType::Named => {
                        FieldIdent::Named(Ident::new("__marker", Span::call_site()))
                    }
                    StructType::Unnamed => FieldIdent::Indexed(Index::from(0)),
                    StructType::Unit => unreachable!(),
                },
                parse_quote! {
                    ::core::marker::PhantomData<&#deserialize_lifetime ()>
                },
            )));

        let mut generics = self.ast.generics.clone();
        generics.params.insert(
            0,
            syn::GenericParam::Lifetime(LifetimeParam::new((*deserialize_lifetime).to_owned())),
        );

        Ok(syn::parse2(struct_definition_expr(
            builder_ident,
            // Builder only needs lifetime if there are groups
            Some(&generics),
            value_expressions_constructors,
            &Self::constructor_type(self.ast),
            &self.ast.vis,
        ))?)
    }

    fn builder_constructor(&self, builder_ident: &Ident) -> Result<Vec<Stmt>, DeriveError> {
        let local_value_expressions_constructors = attribute_fields(self.ast)?
            .into_iter()
            .map(|FieldWithOpts { field_ident, .. }| {
                let expression = quote! {
                    ::core::option::Option::None
                };
                (field_ident, expression)
            })
            .chain(element_fields(self.ast)?.into_iter().map(
                |FieldWithOpts { field_ident, .. }| {
                    let expression = quote! {
                        ::core::option::Option::None
                    };
                    (field_ident, expression)
                },
            ));
        let group_value_expressions_constructors = group_fields(self.ast)?.into_iter().map(
            |FieldWithOpts {
                 field_ident,
                 field_type,
                 ..
             }| {
                let expression = quote! {
                    <#field_type as ::xmlity::de::DeserializationGroup>::builder()
                };

                (field_ident, expression)
            },
        );

        let value_expressions_constructors = local_value_expressions_constructors
            .chain(group_value_expressions_constructors)
            .chain(iter::once((
                match Self::constructor_type(self.ast) {
                    StructType::Named => {
                        FieldIdent::Named(Ident::new("__marker", Span::call_site()))
                    }
                    StructType::Unnamed => FieldIdent::Indexed(Index::from(0)),
                    StructType::Unit => unreachable!(),
                },
                quote! {
                    ::core::marker::PhantomData
                },
            )));

        let expr = constructor_expr(
            builder_ident,
            value_expressions_constructors,
            &Self::constructor_type(self.ast),
        );

        Ok(parse_quote!(#expr))
    }

    fn ident(&self) -> std::borrow::Cow<'_, Ident> {
        Cow::Borrowed(&self.ast.ident)
    }

    fn generics(&self) -> std::borrow::Cow<'_, syn::Generics> {
        Cow::Borrowed(&self.ast.generics)
    }
}

fn finish_constructor_expr<T: quote::ToTokens>(
    ident: T,
    element_fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, ChildOpts>>,
    attribute_fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, AttributeOpts>>,
    group_fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, GroupOpts>>,
    constructor_type: &StructType,
) -> proc_macro2::TokenStream {
    let local_value_expressions_constructors = attribute_fields.into_iter()
        .map(|a: FieldWithOpts<FieldIdent, AttributeOpts>| (
            a.field_ident,
            a.options.should_unwrap_default()
        ))
        .chain(element_fields.into_iter().map(|a: FieldWithOpts<FieldIdent, ChildOpts>| (
            a.field_ident,
            a.options.should_unwrap_default()
        )))
      .map(|( field_ident, should_unwrap_default )| {
          let expression = if should_unwrap_default {
              quote! {
                  ::core::option::Option::unwrap_or_default(self.#field_ident)
              }
          } else {
              quote! {
                  ::core::option::Option::ok_or(self.#field_ident, ::xmlity::de::Error::missing_field(stringify!(#field_ident)))?
              }
          };
          (field_ident, expression)
      });
    let group_value_expressions_constructors =
        group_fields
            .into_iter()
            .map(|FieldWithOpts { field_ident, .. }| {
                let expression = quote! {
                    ::xmlity::de::DeserializationGroupBuilder::finish::<E>(self.#field_ident)?
                };

                (field_ident, expression)
            });

    let value_expressions_constructors =
        local_value_expressions_constructors.chain(group_value_expressions_constructors);

    constructor_expr(ident, value_expressions_constructors, constructor_type)
}

enum DeserializationGroupOption {
    Group(RootGroupOpts),
}

impl DeserializationGroupOption {
    pub fn parse(ast: &DeriveInput) -> Result<Self, DeriveError> {
        let group_opts = RootGroupOpts::parse(ast)?.unwrap_or_default();

        Ok(DeserializationGroupOption::Group(group_opts))
    }
}

pub struct DeriveDeserializationGroup;

impl DeriveMacro for DeriveDeserializationGroup {
    fn input_to_derive(ast: &DeriveInput) -> Result<proc_macro2::TokenStream, DeriveError> {
        let DeserializationGroupOption::Group(opts) = DeserializationGroupOption::parse(ast)?;

        match &ast.data {
            syn::Data::Struct(_) => DeriveDeserializationGroupStruct::new(ast, &opts)
                .total_impl()
                .map(|items| quote! { #(#items)* }),
            syn::Data::Enum(_) => Err(DeriveError::custom(
                "Enums are not supported for deserialization groups.",
            )),
            syn::Data::Union(_) => Err(DeriveError::custom(
                "Unions are not supported for deserialization groups.",
            )),
        }
    }
}
