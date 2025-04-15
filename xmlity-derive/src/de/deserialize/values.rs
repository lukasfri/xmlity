use proc_macro2::Span;
use syn::{
    parse_quote, DataEnum, DeriveInput, ExprIf, Ident, Lifetime, LifetimeParam, Stmt, Variant,
};

use crate::{
    de::common::{DeserializeBuilder, VisitorBuilder, VisitorBuilderExt},
    options::XmlityRootValueDeriveOpts,
    DeriveError,
};

pub struct EnumValueVisitorBuilder<'a> {
    opts: &'a XmlityRootValueDeriveOpts,
}

impl<'a> EnumValueVisitorBuilder<'a> {
    pub fn new(opts: &'a XmlityRootValueDeriveOpts) -> Self {
        Self { opts }
    }

    fn str_value_body(
        &self,
        ast: &syn::DeriveInput,
        value_ident: &Ident,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let DeriveInput { ident, data, .. } = ast;
        let syn::Data::Enum(DataEnum { variants, .. }) = data else {
            panic!("This is guaranteed by the caller");
        };

        let variants = variants.clone().into_iter().map::<Option<ExprIf>, _>(|Variant {
          ident: variant_ident,
          fields: variant_fields,
          ..
      }| {
          let variant_ident_string = self.opts.rename_all.apply_to_variant(&variant_ident.to_string());
          match variant_fields {
              syn::Fields::Named(_fields) => {
                  None
              }
              syn::Fields::Unnamed(_fields) => {
                  None
              }
              syn::Fields::Unit =>{
                  Some(parse_quote! {
                      if ::core::primitive::str::trim(::core::ops::Deref::deref(&#value_ident)) == #variant_ident_string {
                          return ::core::result::Result::Ok(#ident::#variant_ident);
                      }
                  })
              },
          }
      }).collect::<Option<Vec<_>>>();

        let variants = match variants {
            Some(variants) => variants,
            None => return Ok(None),
        };

        let ident_string = ident.to_string();

        Ok(Some(parse_quote! {
            #(#variants)*

            ::core::result::Result::Err(::xmlity::de::Error::no_possible_variant(#ident_string))
        }))
    }
}

impl VisitorBuilder for EnumValueVisitorBuilder<'_> {
    fn visit_text_fn_body(
        &self,
        ast: &syn::DeriveInput,
        _visitor_lifetime: &Lifetime,
        value_ident: &Ident,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let str_ident = Ident::new("__value_str", Span::mixed_site());

        let str_body = self.str_value_body(ast, &str_ident)?;

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
        ast: &syn::DeriveInput,
        _visitor_lifetime: &Lifetime,
        value_ident: &Ident,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let str_ident = Ident::new("__value_str", Span::mixed_site());

        let str_body = self.str_value_body(ast, &str_ident)?;

        let Some(str_body) = str_body else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            let #str_ident = ::xmlity::de::XmlCData::as_str(&#value_ident);
            #(#str_body)*
        }))
    }

    fn visitor_definition(&self, ast: &syn::DeriveInput) -> Result<syn::ItemStruct, DeriveError> {
        let DeriveInput {
            ident, generics, ..
        } = ast;
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
}

impl DeserializeBuilder for EnumValueVisitorBuilder<'_> {
    fn deserialize_fn_body(
        &self,
        ast: &syn::DeriveInput,
        deserializer_ident: &Ident,
        _deserialize_lifetime: &Lifetime,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let formatter_expecting = format!("enum {}", ast.ident);

        let visitor_ident = Ident::new("__Visitor", Span::mixed_site());

        let visitor_def = self.visitor_definition(ast)?;
        let visitor_trait_impl =
            self.visitor_trait_impl(ast, &visitor_ident, &formatter_expecting)?;

        Ok(parse_quote! {
            #visitor_def

            #visitor_trait_impl

            ::xmlity::de::Deserializer::deserialize_any(#deserializer_ident, #visitor_ident {
                lifetime: ::core::marker::PhantomData,
                marker: ::core::marker::PhantomData,
            })
        })
    }
}
