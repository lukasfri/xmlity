use std::{borrow::Cow, ops::Deref};

use proc_macro2::Span;
use syn::{parse_quote, Data, DeriveInput, Ident, Lifetime, LifetimeParam, Stmt};

use crate::{
    de::common::{DeserializeBuilder, VisitorBuilder, VisitorBuilderExt},
    options::{WithExpandedNameExt, XmlityRootAttributeDeriveOpts},
    DeriveError,
};

pub struct StructAttributeVisitorBuilder<'a> {
    opts: &'a XmlityRootAttributeDeriveOpts,
    ast: &'a syn::DeriveInput,
}

impl<'a> StructAttributeVisitorBuilder<'a> {
    pub fn new(opts: &'a XmlityRootAttributeDeriveOpts, ast: &'a syn::DeriveInput) -> Self {
        Self { opts, ast }
    }
}

impl VisitorBuilder for StructAttributeVisitorBuilder<'_> {
    fn visit_attribute_fn_body(
        &self,
        visitor_lifetime: &Lifetime,
        attribute_access_ident: &Ident,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let DeriveInput { ident, data, .. } = &self.ast;

        let Data::Struct(data_struct) = data else {
            unreachable!()
        };
        let XmlityRootAttributeDeriveOpts {
            deserialize_any_name,
            ..
        } = self.opts;
        let ident_name = ident.to_string();
        let expanded_name = if *deserialize_any_name {
            None
        } else {
            Some(self.opts.expanded_name(&ident_name))
        };

        let xml_name_identification = expanded_name.map::<Stmt, _>(|qname| {
              parse_quote! {
                  ::xmlity::de::AttributeAccessExt::ensure_name::<<A as ::xmlity::de::AttributeAccess<#visitor_lifetime>>::Error>(&#attribute_access_ident, &#qname)?;
              }
          });

        let deserialization_impl: Vec<Stmt> = match &data_struct.fields {
              syn::Fields::Unnamed(fields) if fields.unnamed.len() != 1 => return Err(DeriveError::custom("Only tuple structs with 1 element are supported")),
              syn::Fields::Unnamed(_) => {
                  parse_quote! {
                      ::core::str::FromStr::from_str(::xmlity::de::AttributeAccess::value(&#attribute_access_ident))
                          .map(#ident)
                          .map_err(::xmlity::de::Error::custom)
                  }
              }
              syn::Fields::Named(_) =>
              return Err(DeriveError::custom("Named fields in structs are not supported. Only tuple structs with 1 element are supported")),
              syn::Fields::Unit =>
                  return Err(DeriveError::custom("Unit structs are not supported. Only tuple structs with 1 element are supported")),
          };

        Ok(Some(parse_quote! {
            #xml_name_identification

            #(#deserialization_impl)*
        }))
    }

    fn visitor_definition(&self) -> Result<syn::ItemStruct, DeriveError> {
        let ident = self.visitor_ident();
        let generics = self.visitor_generics();

        let non_bound_generics = crate::non_bound_generics(generics.deref());

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
        Cow::Borrowed(&self.ast.ident)
    }

    fn visitor_generics(&self) -> Cow<'_, syn::Generics> {
        Cow::Borrowed(&self.ast.generics)
    }
}

impl DeserializeBuilder for StructAttributeVisitorBuilder<'_> {
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
        Cow::Borrowed(&self.ast.ident)
    }

    fn generics(&self) -> Cow<'_, syn::Generics> {
        Cow::Borrowed(&self.ast.generics)
    }
}
