use proc_macro2::Span;
use syn::{
    parse_quote, DeriveInput, Ident, ImplItemFn, ItemImpl, ItemStruct, Lifetime, LifetimeParam,
    Stmt,
};

use crate::DeriveError;

pub trait VisitorBuilder {
    fn visit_text_fn_body(
        &self,
        ast: &syn::DeriveInput,
        visitor_lifetime: &Lifetime,
        value_ident: &Ident,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let _ = ast;
        let _ = visitor_lifetime;
        let _ = value_ident;
        Ok(None)
    }

    fn visit_cdata_fn_body(
        &self,
        ast: &syn::DeriveInput,
        visitor_lifetime: &Lifetime,
        value_ident: &Ident,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let _ = ast;
        let _ = visitor_lifetime;
        let _ = value_ident;
        Ok(None)
    }

    fn visit_element_fn_body(
        &self,
        ast: &syn::DeriveInput,
        visitor_lifetime: &Lifetime,
        element_access_ident: &Ident,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let _ = ast;
        let _ = visitor_lifetime;
        let _ = element_access_ident;
        Ok(None)
    }

    fn visit_attribute_fn_body(
        &self,
        ast: &syn::DeriveInput,
        visitor_lifetime: &Lifetime,
        attribute_access_ident: &Ident,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let _ = ast;
        let _ = visitor_lifetime;
        let _ = attribute_access_ident;
        Ok(None)
    }

    fn visit_seq_fn_body(
        &self,
        ast: &syn::DeriveInput,
        visitor_lifetime: &Lifetime,
        seq_access_ident: &Ident,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let _ = ast;
        let _ = visitor_lifetime;
        let _ = seq_access_ident;
        Ok(None)
    }

    fn visit_pi_fn_body(
        &self,
        ast: &syn::DeriveInput,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let _ = ast;
        let _ = visitor_lifetime;
        Ok(None)
    }

    fn visit_decl_fn_body(
        &self,
        ast: &syn::DeriveInput,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let _ = ast;
        let _ = visitor_lifetime;
        Ok(None)
    }

    fn visit_comment_fn_body(
        &self,
        ast: &syn::DeriveInput,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let _ = ast;
        let _ = visitor_lifetime;
        Ok(None)
    }

    fn visit_doctype_fn_body(
        &self,
        ast: &syn::DeriveInput,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let _ = ast;
        let _ = visitor_lifetime;
        Ok(None)
    }

    fn visit_none_fn_body(
        &self,
        ast: &syn::DeriveInput,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let _ = ast;
        let _ = visitor_lifetime;
        Ok(None)
    }

    fn definition(&self, ast: &syn::DeriveInput) -> Result<ItemStruct, DeriveError>;
}

pub trait VisitorBuilderExt: VisitorBuilder {
    fn visit_text_fn(
        &self,
        ast: &syn::DeriveInput,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError>;

    fn visit_cdata_fn(
        &self,
        ast: &syn::DeriveInput,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError>;

    fn visit_element_fn(
        &self,
        ast: &syn::DeriveInput,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError>;

    fn visit_attribute_fn(
        &self,
        ast: &syn::DeriveInput,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError>;

    fn visit_seq_fn(
        &self,
        ast: &syn::DeriveInput,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError>;

    fn visit_pi_fn(
        &self,
        ast: &syn::DeriveInput,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError>;

    fn visit_decl_fn(
        &self,
        ast: &syn::DeriveInput,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError>;

    fn visit_comment_fn(
        &self,
        ast: &syn::DeriveInput,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError>;

    fn visit_doctype_fn(
        &self,
        ast: &syn::DeriveInput,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError>;

    fn visit_none_fn(
        &self,
        ast: &syn::DeriveInput,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError>;

    fn visitor_trait_impl(
        &self,
        ast: &syn::DeriveInput,
        visitor_ident: &Ident,
        formatter_expecting: &str,
    ) -> Result<ItemImpl, DeriveError>;
}

impl<T: VisitorBuilder> VisitorBuilderExt for T {
    fn visit_text_fn(
        &self,
        ast: &syn::DeriveInput,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError> {
        let value_ident = Ident::new("__value", Span::mixed_site());

        let body = self.visit_text_fn_body(ast, visitor_lifetime, &value_ident)?;

        let Some(body) = body else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            fn visit_text<E, V>(self, #value_ident: V) -> ::core::result::Result<Self::Value, E>
            where
                E: ::xmlity::de::Error,
                V: ::xmlity::de::XmlText,
            {
                #(#body)*
            }
        }))
    }

    fn visit_cdata_fn(
        &self,
        ast: &syn::DeriveInput,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError> {
        let value_ident = Ident::new("__value", Span::mixed_site());

        let body = self.visit_cdata_fn_body(ast, visitor_lifetime, &value_ident)?;

        let Some(body) = body else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            fn visit_cdata<E, V>(self, #value_ident: V) -> ::core::result::Result<Self::Value, E>
            where
                E: ::xmlity::de::Error,
                V: ::xmlity::de::XmlCData,
            {
                #(#body)*
            }
        }))
    }

    fn visit_element_fn(
        &self,
        ast: &syn::DeriveInput,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError> {
        let element_access_ident = Ident::new("__element_access", Span::mixed_site());

        let body = self.visit_element_fn_body(ast, visitor_lifetime, &element_access_ident)?;

        let Some(body) = body else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            fn visit_element<A>(self, mut #element_access_ident: A) -> ::core::result::Result<Self::Value, <A as ::xmlity::de::AttributesAccess<#visitor_lifetime>>::Error>
            where
                A: xmlity::de::ElementAccess<#visitor_lifetime>,
            {
                #(#body)*
            }
        }))
    }

    fn visit_attribute_fn(
        &self,
        ast: &syn::DeriveInput,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError> {
        let attribute_access_ident = Ident::new("__attribute_access", Span::mixed_site());

        let body = self.visit_attribute_fn_body(ast, visitor_lifetime, &attribute_access_ident)?;

        let Some(body) = body else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            fn visit_attribute<A>(self, #attribute_access_ident: A) -> ::core::result::Result<Self::Value, <A as ::xmlity::de::AttributeAccess<#visitor_lifetime>>::Error>
            where
                A: ::xmlity::de::AttributeAccess<#visitor_lifetime>,
            {
                #(#body)*
            }
        }))
    }

    fn visit_seq_fn(
        &self,
        ast: &syn::DeriveInput,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError> {
        let seq_access_ident = Ident::new("__seq_access", Span::mixed_site());

        let body = self.visit_seq_fn_body(ast, visitor_lifetime, &seq_access_ident)?;

        let Some(body) = body else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            fn visit_seq<S>(self, mut #seq_access_ident: S) -> ::core::result::Result<Self::Value, <S as ::xmlity::de::SeqAccess<#visitor_lifetime>>::Error>
            where
                S: ::xmlity::de::SeqAccess<#visitor_lifetime>,
            {
                #(#body)*
            }
        }))
    }

    fn visit_pi_fn(
        &self,
        ast: &syn::DeriveInput,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError> {
        let value_ident = Ident::new("__value", Span::mixed_site());

        let body = self.visit_pi_fn_body(ast, visitor_lifetime)?;

        let Some(body) = body else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            fn visit_pi<E, V: AsRef<[u8]>>(self, #value_ident: V) -> Result<Self::Value, E>
            where
                E: Error,
            {
                #(#body)*
            }
        }))
    }

    fn visit_decl_fn(
        &self,
        ast: &syn::DeriveInput,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError> {
        let version_ident = Ident::new("__version", Span::mixed_site());
        let encoding_ident = Ident::new("__encoding", Span::mixed_site());
        let standalone_ident = Ident::new("__standalone", Span::mixed_site());

        let body = self.visit_decl_fn_body(ast, visitor_lifetime)?;

        let Some(body) = body else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            fn visit_decl<E, V: AsRef<[u8]>>(
                self,
                #version_ident: V,
                #encoding_ident: Option<V>,
                #standalone_ident: Option<V>,
            ) -> Result<Self::Value, E>
            where
                E: Error,
            {
                #(#body)*
            }
        }))
    }

    fn visit_comment_fn(
        &self,
        ast: &syn::DeriveInput,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError> {
        let value_ident = Ident::new("__value", Span::mixed_site());

        let body = self.visit_comment_fn_body(ast, visitor_lifetime)?;

        let Some(body) = body else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            fn visit_comment<E, V: AsRef<[u8]>>(self, #value_ident: V) -> Result<Self::Value, E>
            where
                E: Error,
            {
                #(#body)*
            }
        }))
    }

    fn visit_doctype_fn(
        &self,
        ast: &syn::DeriveInput,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError> {
        let value_ident = Ident::new("__value", Span::mixed_site());

        let body = self.visit_doctype_fn_body(ast, visitor_lifetime)?;

        let Some(body) = body else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            fn visit_doctype<E, V: AsRef<[u8]>>(self, #value_ident: V) -> Result<Self::Value, E>
            where
                E: Error,
            {
                #(#body)*
            }
        }))
    }

    fn visit_none_fn(
        &self,
        ast: &syn::DeriveInput,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError> {
        let body = self.visit_none_fn_body(ast, visitor_lifetime)?;

        let Some(body) = body else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: Error,
            {
                #(#body)*
            }
        }))
    }

    fn visitor_trait_impl(
        &self,
        ast: &syn::DeriveInput,
        visitor_ident: &Ident,
        formatter_expecting: &str,
    ) -> Result<ItemImpl, DeriveError> {
        let DeriveInput {
            ident, generics, ..
        } = ast;
        let visitor_lifetime = Lifetime::new("'__visitor", Span::mixed_site());

        let visit_text_fn = self.visit_text_fn(ast, &visitor_lifetime)?;
        let visit_cdata_fn = self.visit_cdata_fn(ast, &visitor_lifetime)?;
        let visit_element_fn = self.visit_element_fn(ast, &visitor_lifetime)?;
        let visit_attribute_fn = self.visit_attribute_fn(ast, &visitor_lifetime)?;
        let visit_seq_fn = self.visit_seq_fn(ast, &visitor_lifetime)?;
        let visit_comment_fn = self.visit_comment_fn(ast, &visitor_lifetime)?;
        let visit_pi_fn = self.visit_pi_fn(ast, &visitor_lifetime)?;
        let visit_decl_fn = self.visit_decl_fn(ast, &visitor_lifetime)?;
        let visit_doctype_fn = self.visit_doctype_fn(ast, &visitor_lifetime)?;
        let visit_none_fn = self.visit_none_fn(ast, &visitor_lifetime)?;
        let non_bound_generics = crate::non_bound_generics(generics);

        let mut deserialize_generics = (*generics).to_owned();

        deserialize_generics.params.insert(
            0,
            syn::GenericParam::Lifetime(LifetimeParam::new((visitor_lifetime).clone())),
        );
        let non_bound_deserialize_generics = crate::non_bound_generics(&deserialize_generics);

        Ok(parse_quote! {
            impl #deserialize_generics ::xmlity::de::Visitor<#visitor_lifetime> for #visitor_ident #non_bound_deserialize_generics {
                type Value = #ident #non_bound_generics;
                fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    ::core::fmt::Formatter::write_str(formatter, #formatter_expecting)
                }
                #visit_text_fn
                #visit_cdata_fn
                #visit_element_fn
                #visit_attribute_fn
                #visit_seq_fn
                #visit_comment_fn
                #visit_pi_fn
                #visit_decl_fn
                #visit_doctype_fn
                #visit_none_fn
            }
        })
    }
}

pub trait DeserializeBuilder {
    /// Returns the content inside the `Deserialize::deserialize` function.
    fn deserialize_fn_body(
        &self,
        ast: &syn::DeriveInput,
        deserializer_ident: &Ident,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Vec<Stmt>, DeriveError>;
}

pub trait DeserializeBuilderExt: DeserializeBuilder {
    fn deserialize_fn(
        &self,
        ast: &syn::DeriveInput,
        deserialize_lifetime: &Lifetime,
    ) -> Result<ImplItemFn, DeriveError>;

    fn deserialize_trait_impl(&self, ast: &syn::DeriveInput) -> Result<ItemImpl, DeriveError>;
}

impl<T: DeserializeBuilder> DeserializeBuilderExt for T {
    fn deserialize_fn(
        &self,
        ast: &syn::DeriveInput,
        deserialize_lifetime: &Lifetime,
    ) -> Result<ImplItemFn, DeriveError> {
        let deserializer_ident = Ident::new("__deserializer", Span::mixed_site());
        let body = self.deserialize_fn_body(ast, &deserializer_ident, deserialize_lifetime)?;
        Ok(parse_quote! {
                fn deserialize<D>(#deserializer_ident: D) -> Result<Self, <D as ::xmlity::Deserializer<#deserialize_lifetime>>::Error>
                where
                    D: ::xmlity::Deserializer<#deserialize_lifetime>,
                {
                    #(#body)*
                }

        })
    }

    fn deserialize_trait_impl(
        &self,
        ast @ DeriveInput {
            ident, generics, ..
        }: &syn::DeriveInput,
    ) -> Result<ItemImpl, DeriveError> {
        let deserialize_lifetime = Lifetime::new("'__deserialize", ident.span());

        let non_bound_generics = crate::non_bound_generics(generics);

        let mut deserialize_generics = (*generics).to_owned();

        deserialize_generics.params.insert(
            0,
            syn::GenericParam::Lifetime(LifetimeParam::new((deserialize_lifetime).clone())),
        );

        let deserialize_fn = self.deserialize_fn(ast, &deserialize_lifetime)?;

        Ok(parse_quote! {
            impl #deserialize_generics ::xmlity::Deserialize<#deserialize_lifetime> for #ident #non_bound_generics  {
                #deserialize_fn
            }
        })
    }
}
