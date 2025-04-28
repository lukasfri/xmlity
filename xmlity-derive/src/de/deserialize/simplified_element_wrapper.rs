#![allow(clippy::type_complexity)]

use std::borrow::Cow;

use proc_macro2::Span;
use syn::{parse_quote, ExprWhile, Ident, Lifetime, LifetimeParam, Stmt, Type};

use crate::{
    de::common::{DeserializeBuilder, VisitorBuilder, VisitorBuilderExt},
    DeriveError, ExpandedName,
};

pub struct SingleChildDeserializeElementBuilder<'a> {
    pub ident: &'a syn::Ident,
    pub generics: &'a syn::Generics,
    pub required_expanded_name: Option<ExpandedName<'static>>,
    pub item_type: &'a syn::Type,
    pub extendable: bool,
}

impl SingleChildDeserializeElementBuilder<'_> {
    #[allow(clippy::too_many_arguments)]
    fn visit_element_data_fn_impl(
        ident: &Ident,
        _visitor_lifetime: &syn::Lifetime,
        element_access_ident: &Ident,
        _access_type: &Type,
        item_type: &Type,
        extendable: bool,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let children_access_ident = Ident::new("__children", element_access_ident.span());
        let value_access_ident = Ident::new("__value", element_access_ident.span());

        let extendable_loop: Option<ExprWhile> = if extendable {
            let loop_temporary_value_ident = Ident::new("__vv", Span::call_site());
            Some(parse_quote! {
                while let Some(#loop_temporary_value_ident) = ::xmlity::de::SeqAccess::next_element_seq::<#item_type>(&mut #children_access_ident)? {
                    ::core::iter::Extend::extend(&mut #value_access_ident, [#loop_temporary_value_ident]);
                }
            })
        } else {
            None
        };

        Ok(parse_quote! {
            let mut #children_access_ident = ::xmlity::de::ElementAccess::children(#element_access_ident)?;

            let mut #value_access_ident = ::core::option::Option::ok_or_else(::xmlity::de::SeqAccess::next_element_seq::<#item_type>(
                &mut #children_access_ident,
            )?, ::xmlity::de::Error::missing_data)?;

            #extendable_loop

           ::core::result::Result::Ok(#ident {#value_access_ident })
        })
    }
}

impl VisitorBuilder for SingleChildDeserializeElementBuilder<'_> {
    fn visit_element_fn_body(
        &self,
        visitor_lifetime: &Lifetime,
        element_access_ident: &Ident,
        access_type: &Type,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let xml_name_identification = self.required_expanded_name.as_ref().map::<Stmt, _>(|qname| {
          parse_quote! {
              ::xmlity::de::ElementAccessExt::ensure_name::<<#access_type as ::xmlity::de::AttributesAccess<#visitor_lifetime>>::Error>(&#element_access_ident, &#qname)?;
          }
      });

        let deserialization_impl = Self::visit_element_data_fn_impl(
            self.ident,
            visitor_lifetime,
            element_access_ident,
            access_type,
            self.item_type,
            self.extendable,
        )?;

        Ok(Some(parse_quote! {
            #xml_name_identification

            #(#deserialization_impl)*
        }))
    }

    fn visitor_definition(&self) -> Result<syn::ItemStruct, DeriveError> {
        let Self {
            ident, generics, ..
        } = &self;
        let non_bound_generics = crate::non_bound_generics(generics);

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

impl DeserializeBuilder for SingleChildDeserializeElementBuilder<'_> {
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
