use std::borrow::Cow;

use proc_macro2::Span;
use syn::{parse_quote, Ident, Lifetime, Stmt};

use crate::{
    de::common::{DeserializeBuilder, VisitorBuilder, VisitorBuilderExt},
    DeriveError,
};

pub struct StringLiteralDeserializeBuilder<'a> {
    value: &'a str,
    ident: Ident,
}

impl<'a> StringLiteralDeserializeBuilder<'a> {
    pub fn new(ident: Ident, value: &'a str) -> Self {
        Self {
            ident: ident.clone(),
            value,
        }
    }

    fn str_value_body(&self, value_ident: &Ident) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let ident = &self.ident;
        let value = self.value;
        Ok(Some(parse_quote! {
            if ::core::primitive::str::trim(::core::ops::Deref::deref(&#value_ident)) == #value {
                return ::core::result::Result::Ok(#ident);
            }

            ::core::result::Result::Err(::xmlity::de::Error::no_possible_variant("Value"))
        }))
    }

    pub fn definition(&self) -> Result<syn::ItemStruct, DeriveError> {
        let ident = &self.ident;
        Ok(parse_quote! {
            struct #ident;
        })
    }
}

impl VisitorBuilder for StringLiteralDeserializeBuilder<'_> {
    fn visit_text_fn_body(
        &self,
        _visitor_lifetime: &Lifetime,
        value_ident: &Ident,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let str_ident = Ident::new("__value_str", Span::mixed_site());

        let str_body = self.str_value_body(&str_ident)?;

        let Some(str_body) = str_body else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            let #str_ident = ::xmlity::de::XmlText::as_str(&#value_ident);
            #(#str_body)*
        }))
    }

    fn visit_cdata_fn_body(
        &self,
        _visitor_lifetime: &Lifetime,
        value_ident: &Ident,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let str_ident = Ident::new("__value_str", Span::mixed_site());

        let str_body = self.str_value_body(&str_ident)?;

        let Some(str_body) = str_body else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            let #str_ident = ::xmlity::de::XmlCData::as_str(&#value_ident);
            #(#str_body)*
        }))
    }

    fn visitor_definition(&self) -> Result<syn::ItemStruct, DeriveError> {
        let visitor_ident = Ident::new("__Visitor", Span::mixed_site());
        let visitor_lifetime = Lifetime::new("'__visitor", Span::mixed_site());

        Ok(parse_quote! {
            struct #visitor_ident <#visitor_lifetime> {
                lifetime: ::core::marker::PhantomData<&#visitor_lifetime ()>,
            }
        })
    }

    fn visitor_ident(&self) -> Cow<'_, Ident> {
        Cow::Borrowed(&self.ident)
    }

    fn visitor_generics(&self) -> Cow<'_, syn::Generics> {
        Cow::Owned(syn::Generics::default())
    }
}

impl DeserializeBuilder for StringLiteralDeserializeBuilder<'_> {
    fn deserialize_fn_body(
        &self,
        deserializer_ident: &Ident,
        _deserialize_lifetime: &Lifetime,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let formatter_expecting = format!("literal {}", self.value);

        let visitor_ident = Ident::new("__Visitor", Span::mixed_site());

        let visitor_def = self.visitor_definition()?;
        let visitor_trait_impl = self.visitor_trait_impl(&visitor_ident, &formatter_expecting)?;

        Ok(parse_quote! {
            #visitor_def

            #visitor_trait_impl

            ::xmlity::de::Deserializer::deserialize_any(#deserializer_ident, #visitor_ident {
                lifetime: ::core::marker::PhantomData,
            })
        })
    }

    fn ident(&self) -> Cow<'_, Ident> {
        Cow::Borrowed(&self.ident)
    }

    fn generics(&self) -> Cow<'_, syn::Generics> {
        Cow::Owned(syn::Generics::default())
    }
}
