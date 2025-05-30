#![allow(clippy::type_complexity)]

use std::borrow::Cow;

use proc_macro2::Span;
use quote::quote;
use syn::{parse_quote, Expr, Ident, Lifetime, LifetimeParam, Stmt, Type};

use crate::{
    common::{
        constructor_expr, non_bound_generics, ExpandedName, FieldIdent, StructType,
        StructTypeWithFields,
    },
    de::{
        builders::{DeserializeBuilder, VisitorBuilder, VisitorBuilderExt},
        common::{
            all_attributes_done_expr, attribute_done_expr, builder_attribute_field_visitor,
            SeqVisitLoop,
        },
    },
    options::{
        records::fields::{
            AttributeOpts, ChildOpts, FieldAttributeGroupOpts, FieldOpts, FieldValueGroupOpts,
            GroupOpts,
        },
        ElementOrder, FieldWithOpts,
    },
    DeriveError, DeriveResult,
};

use super::RecordInput;

pub struct RecordDeserializeElementBuilder<'a, T: Fn(syn::Expr) -> syn::Expr> {
    pub input: &'a RecordInput<'a, T>,
    pub ignore_whitespace: bool,
    pub required_expanded_name: Option<ExpandedName<'static>>,
    pub allow_unknown_attributes: bool,
    pub allow_unknown_children: bool,
    pub children_order: ElementOrder,
    pub attribute_order: ElementOrder,
}

impl<'a, T: Fn(syn::Expr) -> syn::Expr> RecordDeserializeElementBuilder<'a, T> {
    pub fn field_decl(
        element_fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, ChildOpts>>,
        attribute_fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, AttributeOpts>>,
        group_fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, GroupOpts>>,
    ) -> Vec<Stmt> {
        let getter_declarations = attribute_fields
            .into_iter()
            .map::<Stmt, _>(
                |FieldWithOpts {
                    field_ident,
                     field_type,
                     ..
                 }| {
                    let builder_field_ident = field_ident.to_named_ident();
                    parse_quote! {
                        let mut #builder_field_ident = ::core::option::Option::<#field_type>::None;
                    }
                },
            )
            .chain(element_fields.into_iter().map::<Stmt, _>(
                |FieldWithOpts {
                    field_ident,
                     field_type,
                     ..
                 }| {
                    let builder_field_ident = field_ident.to_named_ident();
                    parse_quote! {
                        let mut #builder_field_ident = ::core::option::Option::<#field_type>::None;
                    }
                },
            )).chain(group_fields.into_iter().map::<Stmt, _>(
                |FieldWithOpts {
                    field_ident,
                     field_type,
                     ..
                 }| {
                    let builder_field_ident = field_ident.to_named_ident();
                    parse_quote! {
                        let mut #builder_field_ident = <#field_type as ::xmlity::de::DeserializationGroup>::builder();
                    }
                },
            ));

        parse_quote! {
            #(#getter_declarations)*
        }
    }

    pub fn constructor_expr(
        ident: &syn::Path,
        visitor_lifetime: &syn::Lifetime,
        access_type: &syn::Type,
        element_fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, ChildOpts>>,
        attribute_fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, AttributeOpts>>,
        group_fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, GroupOpts>>,
        constructor_type: StructType,
    ) -> Expr {
        let local_value_expressions_constructors = attribute_fields.into_iter()
            .map(|a: FieldWithOpts<FieldIdent, AttributeOpts>| (
                a.field_ident,
                a.options.default_or_else()
            ))
            .chain(element_fields.into_iter().map(|a: FieldWithOpts<FieldIdent, ChildOpts>| (
                a.field_ident,
                a.options.default_or_else()
            )))
            .map::<(_, Expr), _>(|(field_ident, default_or_else)| {
                let builder_field_ident = field_ident.to_named_ident();

                let expression = if let Some(default_or_else) = default_or_else {
                    parse_quote! {
                        ::core::option::Option::unwrap_or_else(#builder_field_ident, #default_or_else)
                    }
                } else {
                    parse_quote! {
                        ::core::option::Option::ok_or(#builder_field_ident, ::xmlity::de::Error::missing_field(stringify!(#field_ident)))?
                    }
                };
                (field_ident, expression)
            });
        let group_value_expressions_constructors = group_fields.into_iter().map::<(_, Expr), _>(
            |FieldWithOpts {
                 field_ident,
                 ..
             }| {
                let builder_field_ident = field_ident.to_named_ident();
                let expression = parse_quote! {
                    ::xmlity::de::DeserializationGroupBuilder::finish::<<#access_type as ::xmlity::de::AttributesAccess<#visitor_lifetime>>::Error>(#builder_field_ident)?
                };

                (field_ident, expression)
            },
        );

        let value_expressions_constructors =
            local_value_expressions_constructors.chain(group_value_expressions_constructors);

        constructor_expr(ident, value_expressions_constructors, &constructor_type)
    }

    pub fn attribute_access(
        access_ident: &Ident,
        span: proc_macro2::Span,
        fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, FieldAttributeGroupOpts>> + Clone,
        allow_unknown_attributes: bool,
        order: ElementOrder,
    ) -> DeriveResult<Vec<Stmt>> {
        let field_visits = builder_attribute_field_visitor(
            access_ident,
            quote! {},
            fields.clone(),
            parse_quote! {break;},
            match order {
                ElementOrder::Loose => parse_quote! {break;},
                ElementOrder::None => parse_quote! {continue;},
            },
            parse_quote! {continue;},
            parse_quote! {},
            match order {
                ElementOrder::Loose => true,
                ElementOrder::None => false,
            },
        )?;

        match order {
            ElementOrder::Loose => field_visits.into_iter().zip(fields).map(|(field_visit, field)| {
                let skip_unknown: Vec<Stmt> = if allow_unknown_attributes {
                    let skip_ident = Ident::new("__skip", span);
                    parse_quote! {
                        let #skip_ident = ::xmlity::de::AttributesAccess::next_attribute::<::xmlity::types::utils::IgnoredAny>(&mut #access_ident).unwrap_or(None);
                        if ::core::option::Option::is_none(&#skip_ident) {
                            break;
                        }
                        continue;
                    }
                } else {
                    let condition = attribute_done_expr(field, quote! {});

                    parse_quote! {
                        if #condition {
                            break;
                        } else {
                            return ::core::result::Result::Err(::xmlity::de::Error::unknown_child());
                        }
                    }
                };

                Ok(parse_quote! {
                    loop {
                        #field_visit
                        #(#skip_unknown)*
                    }
                })
            }).collect(),
            ElementOrder::None => {
                let skip_unknown: Vec<Stmt> = if allow_unknown_attributes {
                    let skip_ident = Ident::new("__skip", span);
                    parse_quote! {
                        let #skip_ident = ::xmlity::de::AttributesAccess::next_attribute::<::xmlity::types::utils::IgnoredAny>(&mut #access_ident).unwrap_or(None);
                        if ::core::option::Option::is_none(&#skip_ident) {
                            break;
                        }
                    }
                } else {
                    let all_some_condition = all_attributes_done_expr(fields, quote! {});

                    parse_quote! {
                        if #all_some_condition {
                            break;
                        } else {
                            return ::core::result::Result::Err(::xmlity::de::Error::unknown_child());
                        }
                    }
                };

                Ok(parse_quote! {
                    loop {
                        #(#field_visits)*
                        #(#skip_unknown)*
                    }
                })
            },
        }
    }

    pub fn element_access(
        element_access_ident: &Ident,
        fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, FieldValueGroupOpts>> + Clone,
        allow_unknown_children: bool,
        order: ElementOrder,
        ignore_whitespace: bool,
    ) -> DeriveResult<Vec<Stmt>> {
        let access_ident = Ident::new("__children", element_access_ident.span());

        let visit = SeqVisitLoop::new(
            &access_ident,
            allow_unknown_children,
            order,
            fields,
            ignore_whitespace,
        );

        let field_storage = visit.field_storage();
        let access_loop = visit.access_loop()?;

        Ok(parse_quote! {
            let mut #access_ident = ::xmlity::de::ElementAccess::children(#element_access_ident)?;

            #field_storage

            #(#access_loop)*
        })
    }
}

impl<T: Fn(syn::Expr) -> syn::Expr> VisitorBuilder for RecordDeserializeElementBuilder<'_, T> {
    fn visit_element_fn_body(
        &self,
        visitor_lifetime: &Lifetime,
        element_access_ident: &Ident,
        access_type: &Type,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let Self {
            input,
            ignore_whitespace,
            required_expanded_name,
            allow_unknown_attributes,
            allow_unknown_children,
            attribute_order,
            children_order,
            ..
        } = self;
        let RecordInput {
            impl_for_ident: ident,
            ..
        } = input;

        let xml_name_identification = required_expanded_name.as_ref().map::<Stmt, _>(|qname| {
          parse_quote! {
              ::xmlity::de::ElementAccessExt::ensure_name::<<#access_type as ::xmlity::de::AttributesAccess<#visitor_lifetime>>::Error>(&#element_access_ident, &#qname)?;
          }
      });

        let (constructor_type, fields) = match &input.fields {
            StructTypeWithFields::Named(n) => (
                StructType::Named,
                n.iter()
                    .map(|a| a.clone().map_ident(FieldIdent::Named))
                    .collect(),
            ),
            StructTypeWithFields::Unnamed(n) => (
                StructType::Unnamed,
                n.iter()
                    .map(|a| a.clone().map_ident(FieldIdent::Indexed))
                    .collect(),
            ),
            StructTypeWithFields::Unit => (StructType::Unit, Vec::new()),
        };
        let element_fields = fields.clone().into_iter().filter_map(|field| {
            field.map_options_opt(|opt| match opt {
                FieldOpts::Value(opts) => Some(opts),
                _ => None,
            })
        });

        let attribute_fields = fields.clone().into_iter().filter_map(|field| {
            field.map_options_opt(|opt| match opt {
                FieldOpts::Attribute(opts) => Some(opts),
                _ => None,
            })
        });

        let group_fields = fields.clone().into_iter().filter_map(|field| {
            field.map_options_opt(|opt| match opt {
                FieldOpts::Group(opts) => Some(opts),
                _ => None,
            })
        });

        let getter_declarations = Self::field_decl(
            element_fields.clone(),
            attribute_fields.clone(),
            group_fields.clone(),
        );

        let attribute_group_fields = fields.clone().into_iter().filter_map(|field| {
            field.map_options_opt(|opt| match opt {
                FieldOpts::Attribute(opts) => Some(FieldAttributeGroupOpts::Attribute(opts)),
                FieldOpts::Group(opts) => Some(FieldAttributeGroupOpts::Group(opts)),
                FieldOpts::Value(_) => None,
            })
        });

        let element_group_fields = fields.clone().into_iter().filter_map(|field| {
            field.map_options_opt(|opt| match opt {
                FieldOpts::Value(opts) => Some(FieldValueGroupOpts::Value(opts)),
                FieldOpts::Group(opts) => Some(FieldValueGroupOpts::Group(opts)),
                FieldOpts::Attribute(_) => None,
            })
        });

        let attribute_loop = if attribute_group_fields.clone().next().is_some() {
            Self::attribute_access(
                element_access_ident,
                ident.span(),
                attribute_group_fields,
                *allow_unknown_attributes,
                *attribute_order,
            )?
        } else {
            Vec::new()
        };

        let children_loop = if element_group_fields.clone().next().is_some() {
            Self::element_access(
                element_access_ident,
                element_group_fields,
                *allow_unknown_children,
                *children_order,
                *ignore_whitespace,
            )?
        } else {
            Vec::new()
        };

        let constructor = (self.input.wrapper_function)(Self::constructor_expr(
            &self.input.constructor_path,
            visitor_lifetime,
            access_type,
            element_fields.clone(),
            attribute_fields.clone(),
            group_fields.clone(),
            constructor_type,
        ));

        Ok(Some(parse_quote! {
            #xml_name_identification

            #(#getter_declarations)*

            #(#attribute_loop)*

            #(#children_loop)*

            ::core::result::Result::Ok(#constructor)
        }))
    }

    fn visitor_definition(&self) -> Result<syn::ItemStruct, DeriveError> {
        let RecordInput {
            impl_for_ident: ident,
            generics,
            ..
        } = &self.input;
        let non_bound_generics = non_bound_generics(generics);

        let mut deserialize_generics = generics.as_ref().clone();

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
        Cow::Borrowed(self.input.impl_for_ident.as_ref())
    }

    fn visitor_generics(&self) -> Cow<'_, syn::Generics> {
        Cow::Borrowed(self.input.generics.as_ref())
    }
}

impl<T: Fn(syn::Expr) -> syn::Expr> DeserializeBuilder for RecordDeserializeElementBuilder<'_, T> {
    fn deserialize_fn_body(
        &self,

        deserializer_ident: &Ident,
        _deserialize_lifetime: &Lifetime,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let formatter_expecting = format!("struct {}", self.input.impl_for_ident);

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
        Cow::Borrowed(self.input.impl_for_ident.as_ref())
    }

    fn generics(&self) -> Cow<'_, syn::Generics> {
        Cow::Borrowed(self.input.generics.as_ref())
    }
}
