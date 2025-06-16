#![allow(clippy::type_complexity)]

use std::borrow::Cow;

use proc_macro2::Span;
use syn::{parse_quote, Ident, Lifetime, LifetimeParam, Stmt, Type};

use crate::{
    common::{non_bound_generics, ExpandedName, FieldIdent, RecordInput, StructTypeWithFields},
    de::builders::{DeserializeBuilder, VisitorBuilder, VisitorBuilderExt},
    options::{
        records::fields::{ChildOpts, ElementOpts, FieldOpts, GroupOpts, ValueOpts},
        AllowUnknown, ElementOrder, Extendable, FieldWithOpts, IgnoreWhitespace,
        WithExpandedNameExt,
    },
    DeriveError,
};

use super::elements::RecordDeserializeElementBuilder;

pub struct DeserializeSingleChildElementBuilder<'a> {
    pub ident: &'a syn::Ident,
    pub generics: &'a syn::Generics,
    pub required_expanded_name: Option<ExpandedName<'static>>,
    pub item_type: &'a syn::Type,
    pub extendable: Extendable,
    pub group: bool,
    pub default: bool,
    pub default_with: Option<syn::Path>,
}

impl ElementOpts {
    pub fn to_builder<'a>(
        &'a self,
        field_ident: &'a FieldIdent,
        ident: &'a Ident,
        generics: &'a syn::Generics,
        item_type: &'a Type,
    ) -> DeserializeSingleChildElementBuilder<'a> {
        DeserializeSingleChildElementBuilder {
            ident,
            generics,
            required_expanded_name: Some(
                self.expanded_name(field_ident.to_named_ident().to_string().as_str())
                    .into_owned(),
            ),
            item_type,
            default: self.default,
            default_with: self.default_with.clone(),
            extendable: self.extendable,
            group: self.group,
        }
    }
}

impl DeserializeSingleChildElementBuilder<'_> {
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

    pub fn unwrap_expression(&self) -> impl Fn(&syn::Expr) -> syn::Expr + Clone {
        let value_access_ident = self.value_access_ident();

        move |value_expr: &syn::Expr| {
            parse_quote! {
                #value_expr.#value_access_ident
            }
        }
    }
}

impl VisitorBuilder for DeserializeSingleChildElementBuilder<'_> {
    fn visit_element_fn_body(
        &self,
        visitor_lifetime: &Lifetime,
        element_access_ident: &Ident,
        access_type: &Type,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let ident = self.ident;

        let input = RecordInput {
            impl_for_ident: Cow::Borrowed(self.ident),
            constructor_path: Cow::Owned(parse_quote!(#ident)),
            result_type: Cow::Borrowed(self.item_type),
            generics: Cow::Owned(parse_quote!()),
            wrapper_function: std::convert::identity,
            record_path: Cow::Owned(parse_quote!(self)),
            fields: StructTypeWithFields::Named(vec![FieldWithOpts {
                field_ident: self.value_access_ident(),
                field_type: self.item_type.clone(),
                options: if self.group {
                    FieldOpts::Group(GroupOpts {})
                } else {
                    FieldOpts::Value(ChildOpts::Value(ValueOpts {
                        default: self.default,
                        default_with: self.default_with.clone(),
                        extendable: self.extendable,
                        skip_serializing_if: None,
                    }))
                },
            }]),
            sub_path_ident: None,
            fallable_deconstruction: false,
        };

        let builder = RecordDeserializeElementBuilder {
            input: &input,
            ignore_whitespace: IgnoreWhitespace::default(),
            required_expanded_name: self.required_expanded_name.clone(),
            allow_unknown_attributes: AllowUnknown::default(),
            allow_unknown_children: AllowUnknown::default(),
            children_order: ElementOrder::None,
            attribute_order: ElementOrder::None,
        };

        builder.visit_element_fn_body(visitor_lifetime, element_access_ident, access_type)
    }

    fn visitor_definition(&self) -> Result<syn::ItemStruct, DeriveError> {
        let Self {
            ident, generics, ..
        } = &self;
        let non_bound_generics = non_bound_generics(generics);

        let mut deserialize_generics = (*generics).to_owned();

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

impl DeserializeBuilder for DeserializeSingleChildElementBuilder<'_> {
    fn deserialize_fn_body(
        &self,

        deserializer_ident: &Ident,
        _deserialize_lifetime: &Lifetime,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let formatter_expecting = format!("struct {}", self.ident);

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
