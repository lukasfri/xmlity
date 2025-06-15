#![allow(clippy::type_complexity)]

use std::borrow::Cow;

use proc_macro2::Span;
use syn::{parse_quote, Ident, Lifetime, LifetimeParam, Stmt, Type};

use crate::{
    common::{
        constructor_expr, non_bound_generics, ExpandedName, FieldIdent, StructType,
        StructTypeWithFields,
    },
    de::{
        builders::{DeserializeBuilder, VisitorBuilder, VisitorBuilderExt},
        components::ElementLoopAccessor,
    },
    options::{AllowUnknown, ElementOrder, IgnoreWhitespace},
    DeriveError,
};

use super::RecordInput;

pub struct RecordDeserializeElementBuilder<'a, T: Fn(syn::Expr) -> syn::Expr> {
    pub input: &'a RecordInput<'a, T>,
    pub ignore_whitespace: IgnoreWhitespace,
    pub required_expanded_name: Option<ExpandedName<'static>>,
    pub allow_unknown_attributes: AllowUnknown,
    pub allow_unknown_children: AllowUnknown,
    pub children_order: ElementOrder,
    pub attribute_order: ElementOrder,
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
            required_expanded_name,
            ..
        } = self;

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

        let element_loop_accessor = (!fields.is_empty()).then(|| {
            ElementLoopAccessor::new(
                self.allow_unknown_children,
                self.allow_unknown_attributes,
                self.children_order,
                self.attribute_order,
                self.ignore_whitespace,
            )
        });

        let getter_declarations = element_loop_accessor
            .as_ref()
            .map(|a| a.field_definitions(fields.clone()))
            .transpose()?
            .unwrap_or_default();

        let attribute_loop = element_loop_accessor
            .as_ref()
            .map(|a| {
                a.attribute_access_loop(fields.clone(), &parse_quote!(&mut #element_access_ident))
            })
            .transpose()?
            .unwrap_or_default();

        let children_access_ident = Ident::new("__children", element_access_ident.span());

        let children_loop = element_loop_accessor
            .as_ref()
            .map(|a| {
                a.children_access_loop(fields.clone(), &parse_quote!(&mut #children_access_ident))
            })
            .transpose()?
            .unwrap_or_default();

        let error_type: syn::Type = parse_quote!(
            <#access_type as ::xmlity::de::AttributesAccess<#visitor_lifetime>>::Error
        );

        let constructor_exprs = element_loop_accessor
            .as_ref()
            .map(|a| a.value_expressions(fields, visitor_lifetime, &error_type))
            .transpose()?
            .unwrap_or_default();

        let constructor = (self.input.wrapper_function)(constructor_expr(
            &self.input.constructor_path,
            constructor_exprs,
            &constructor_type,
        ));

        Ok(Some(parse_quote! {
            #xml_name_identification

            #(#getter_declarations)*

            #(#attribute_loop)*

            let mut #children_access_ident = ::xmlity::de::ElementAccess::children(#element_access_ident)?;

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
