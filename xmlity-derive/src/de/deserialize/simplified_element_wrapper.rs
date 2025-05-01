#![allow(clippy::type_complexity)]

use std::borrow::Cow;

use proc_macro2::Span;
use syn::{parse_quote, Expr, ExprWhile, Ident, Lifetime, LifetimeParam, Stmt, Type};

use crate::{
    common::{non_bound_generics, ExpandedName},
    de::builders::{DeserializeBuilder, VisitorBuilder, VisitorBuilderExt},
    options::Extendable,
    DeriveError,
};

pub struct SingleChildDeserializeElementBuilder<'a> {
    pub ident: &'a syn::Ident,
    pub generics: &'a syn::Generics,
    pub required_expanded_name: Option<ExpandedName<'static>>,
    pub item_type: &'a syn::Type,
    pub extendable: Extendable,
}

impl SingleChildDeserializeElementBuilder<'_> {
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
}

impl VisitorBuilder for SingleChildDeserializeElementBuilder<'_> {
    fn visit_element_fn_body(
        &self,
        visitor_lifetime: &Lifetime,
        element_access_ident: &Ident,
        access_type: &Type,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let Self {
            ident,
            item_type,
            extendable,
            required_expanded_name,
            ..
        } = self;

        let xml_name_identification = required_expanded_name.as_ref().map::<Stmt, _>(|qname| {
          parse_quote! {
              ::xmlity::de::ElementAccessExt::ensure_name::<<#access_type as ::xmlity::de::AttributesAccess<#visitor_lifetime>>::Error>(&#element_access_ident, &#qname)?;
          }
      });

        let children_access_ident = Ident::new("__children", element_access_ident.span());
        let value_access_ident = self.value_access_ident();

        let extendable_loop: Option<ExprWhile> = if matches!(
            extendable,
            Extendable::Iterator | Extendable::Single
        ) {
            let loop_temporary_value_ident = Ident::new("__vv", Span::call_site());

            let extendable_value: Expr = if *extendable == Extendable::Iterator {
                parse_quote! {
                    {
                        let mut #loop_temporary_value_ident = ::core::iter::Iterator::peekable(
                            ::core::iter::IntoIterator::into_iter(#loop_temporary_value_ident)
                        );

                        if ::core::option::Option::is_none(&::core::iter::Peekable::peek(&mut #loop_temporary_value_ident)) {
                            break;
                        }

                        #loop_temporary_value_ident
                    }
                }
            } else {
                parse_quote! { [#loop_temporary_value_ident] }
            };
            Some(parse_quote! {
                while let Some(#loop_temporary_value_ident) = ::xmlity::de::SeqAccess::next_element_seq::<#item_type>(&mut #children_access_ident)? {
                    ::core::iter::Extend::extend(&mut #value_access_ident, #extendable_value);
                }
            })
        } else {
            None
        };

        Ok(Some(parse_quote! {
            #xml_name_identification

            let mut #children_access_ident = ::xmlity::de::ElementAccess::children(#element_access_ident)?;

            let mut #value_access_ident = ::core::option::Option::ok_or_else(::xmlity::de::SeqAccess::next_element_seq::<#item_type>(
                &mut #children_access_ident,
            )?, ::xmlity::de::Error::missing_data)?;

            #extendable_loop

           ::core::result::Result::Ok(#ident {#value_access_ident })
        }))
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
