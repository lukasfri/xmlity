use std::{borrow::Cow, ops::Deref};

use proc_macro2::Span;
use syn::{
    parse_quote, Generics, Ident, ImplItemFn, Item, ItemImpl, ItemStruct, Lifetime, LifetimeParam,
    Stmt, Type,
};

use crate::{common::non_bound_generics, DeriveError};

pub trait VisitorBuilder {
    fn visit_text_fn_body(
        &self,
        visitor_lifetime: &Lifetime,
        access_ident: &Ident,
        access_type: &Type,
        error_type: &Type,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let _ = visitor_lifetime;
        let _ = access_ident;
        let _ = access_type;
        let _ = error_type;
        Ok(None)
    }

    fn visit_cdata_fn_body(
        &self,
        visitor_lifetime: &Lifetime,
        access_ident: &Ident,
        access_type: &Type,
        error_type: &Type,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let _ = visitor_lifetime;
        let _ = access_ident;
        let _ = access_type;
        let _ = error_type;
        Ok(None)
    }

    fn visit_element_fn_body(
        &self,
        visitor_lifetime: &Lifetime,
        access_ident: &Ident,
        access_type: &Type,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let _ = visitor_lifetime;
        let _ = access_ident;
        let _ = access_type;
        Ok(None)
    }

    fn visit_attribute_fn_body(
        &self,
        visitor_lifetime: &Lifetime,
        access_ident: &Ident,
        access_type: &Type,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let _ = visitor_lifetime;
        let _ = access_ident;
        let _ = access_type;
        Ok(None)
    }

    fn visit_seq_fn_body(
        &self,
        visitor_lifetime: &Lifetime,
        access_ident: &Ident,
        access_type: &Type,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let _ = visitor_lifetime;
        let _ = access_ident;
        let _ = access_type;
        Ok(None)
    }

    fn visit_pi_fn_body(
        &self,
        visitor_lifetime: &Lifetime,
        access_ident: &Ident,
        access_type: &Type,
        error_type: &Type,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let _ = visitor_lifetime;
        let _ = access_ident;
        let _ = access_type;
        let _ = error_type;
        Ok(None)
    }

    fn visit_decl_fn_body(
        &self,
        visitor_lifetime: &Lifetime,
        version_ident: &Ident,
        encoding_ident: &Ident,
        standalone_ident: &Ident,
        access_type: &Type,
        error_type: &Type,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let _ = visitor_lifetime;
        let _ = version_ident;
        let _ = encoding_ident;
        let _ = standalone_ident;
        let _ = access_type;
        let _ = error_type;
        Ok(None)
    }

    fn visit_comment_fn_body(
        &self,
        visitor_lifetime: &Lifetime,
        access_ident: &Ident,
        access_type: &Type,
        error_type: &Type,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let _ = visitor_lifetime;
        let _ = access_ident;
        let _ = access_type;
        let _ = error_type;
        Ok(None)
    }

    fn visit_doctype_fn_body(
        &self,
        visitor_lifetime: &Lifetime,
        access_ident: &Ident,
        access_type: &Type,
        error_type: &Type,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let _ = visitor_lifetime;
        let _ = access_ident;
        let _ = access_type;
        let _ = error_type;
        Ok(None)
    }

    fn visit_none_fn_body(
        &self,
        visitor_lifetime: &Lifetime,
        error_type: &Type,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let _ = visitor_lifetime;
        let _ = error_type;
        Ok(None)
    }

    fn visitor_definition(&self) -> Result<ItemStruct, DeriveError>;

    fn visitor_ident(&self) -> Cow<'_, Ident>;
    fn visitor_generics(&self) -> Cow<'_, Generics>;
}

pub trait VisitorBuilderExt: VisitorBuilder {
    fn visit_text_fn(&self, visitor_lifetime: &Lifetime)
        -> Result<Option<ImplItemFn>, DeriveError>;

    fn visit_cdata_fn(
        &self,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError>;

    fn visit_element_fn(
        &self,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError>;

    fn visit_attribute_fn(
        &self,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError>;

    fn visit_seq_fn(&self, visitor_lifetime: &Lifetime) -> Result<Option<ImplItemFn>, DeriveError>;

    fn visit_pi_fn(&self, visitor_lifetime: &Lifetime) -> Result<Option<ImplItemFn>, DeriveError>;

    fn visit_decl_fn(&self, visitor_lifetime: &Lifetime)
        -> Result<Option<ImplItemFn>, DeriveError>;

    fn visit_comment_fn(
        &self,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError>;

    fn visit_doctype_fn(
        &self,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError>;

    fn visit_none_fn(&self, visitor_lifetime: &Lifetime)
        -> Result<Option<ImplItemFn>, DeriveError>;

    fn visitor_trait_impl(
        &self,
        visitor_ident: &Ident,
        formatter_expecting: &str,
    ) -> Result<ItemImpl, DeriveError>;
}

impl<T: VisitorBuilder> VisitorBuilderExt for T {
    fn visit_text_fn(
        &self,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError> {
        let access_ident = Ident::new("__value", Span::mixed_site());
        let access_type: Type = parse_quote!(__XmlityAccess);
        let error_type: Type = parse_quote!(__XmlityError);

        let body =
            self.visit_text_fn_body(visitor_lifetime, &access_ident, &access_type, &error_type)?;

        let Some(body) = body else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            fn visit_text<#error_type, #access_type>(self, #access_ident: #access_type) -> ::core::result::Result<Self::Value, #error_type>
            where
                #error_type: ::xmlity::de::Error,
                #access_type: ::xmlity::de::XmlText,
            {
                #(#body)*
            }
        }))
    }

    fn visit_cdata_fn(
        &self,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError> {
        let access_ident = Ident::new("__value", Span::mixed_site());
        let access_type: Type = parse_quote!(__XmlityAccess);
        let error_type: Type = parse_quote!(__XmlityError);

        let body =
            self.visit_cdata_fn_body(visitor_lifetime, &access_ident, &access_type, &error_type)?;

        let Some(body) = body else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            fn visit_cdata<#error_type, #access_type>(self, #access_ident: #access_type) -> ::core::result::Result<Self::Value, #error_type>
            where
                #error_type: ::xmlity::de::Error,
                #access_type: ::xmlity::de::XmlCData,
            {
                #(#body)*
            }
        }))
    }

    fn visit_element_fn(
        &self,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError> {
        let element_access_ident = Ident::new("__element_access", Span::mixed_site());
        let access_type: Type = parse_quote!(___XmlityElementAccess);

        let body =
            self.visit_element_fn_body(visitor_lifetime, &element_access_ident, &access_type)?;

        let Some(body) = body else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            fn visit_element<#access_type>(self, mut #element_access_ident: #access_type) -> ::core::result::Result<Self::Value, <#access_type as ::xmlity::de::AttributesAccess<#visitor_lifetime>>::Error>
            where
            #access_type: xmlity::de::ElementAccess<#visitor_lifetime>,
            {
                #(#body)*
            }
        }))
    }

    fn visit_attribute_fn(
        &self,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError> {
        let attribute_access_ident = Ident::new("__attribute_access", Span::mixed_site());
        let access_type: Type = parse_quote!(___XmlityAttributeAccess);

        let body =
            self.visit_attribute_fn_body(visitor_lifetime, &attribute_access_ident, &access_type)?;

        let Some(body) = body else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            fn visit_attribute<#access_type>(self, #attribute_access_ident: #access_type) -> ::core::result::Result<Self::Value, <#access_type as ::xmlity::de::AttributeAccess<#visitor_lifetime>>::Error>
            where
                #access_type: ::xmlity::de::AttributeAccess<#visitor_lifetime>,
            {
                #(#body)*
            }
        }))
    }

    fn visit_seq_fn(&self, visitor_lifetime: &Lifetime) -> Result<Option<ImplItemFn>, DeriveError> {
        let seq_access_ident = Ident::new("__seq_access", Span::mixed_site());
        let access_type: Type = parse_quote!(__XmlitySeqAccess);

        let body = self.visit_seq_fn_body(visitor_lifetime, &seq_access_ident, &access_type)?;

        let Some(body) = body else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            fn visit_seq<#access_type>(self, mut #seq_access_ident: #access_type) -> ::core::result::Result<Self::Value, <#access_type as ::xmlity::de::SeqAccess<#visitor_lifetime>>::Error>
            where
                #access_type: ::xmlity::de::SeqAccess<#visitor_lifetime>,
            {
                #(#body)*
            }
        }))
    }

    fn visit_pi_fn(&self, visitor_lifetime: &Lifetime) -> Result<Option<ImplItemFn>, DeriveError> {
        let value_ident = Ident::new("__value", Span::mixed_site());
        let access_type: Type = parse_quote!(__XmlityAccess);
        let error_type: Type = parse_quote!(__XmlityError);

        let body =
            self.visit_pi_fn_body(visitor_lifetime, &value_ident, &access_type, &error_type)?;

        let Some(body) = body else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            fn visit_pi<#error_type, #access_type: AsRef<[u8]>>(self, #value_ident: #access_type) -> Result<Self::Value, #error_type>
            where
                #error_type: ::xmlity::de::Error,
            {
                #(#body)*
            }
        }))
    }

    fn visit_decl_fn(
        &self,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError> {
        let version_ident = Ident::new("__version", Span::mixed_site());
        let encoding_ident = Ident::new("__encoding", Span::mixed_site());
        let standalone_ident = Ident::new("__standalone", Span::mixed_site());
        let access_type: Type = parse_quote!(__XmlityAccess);
        let error_type: Type = parse_quote!(__XmlityError);

        let body = self.visit_decl_fn_body(
            visitor_lifetime,
            &version_ident,
            &encoding_ident,
            &standalone_ident,
            &access_type,
            &error_type,
        )?;

        let Some(body) = body else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            fn visit_decl<#error_type, #access_type: AsRef<[u8]>>(
                self,
                #version_ident: #access_type,
                #encoding_ident: Option<#access_type>,
                #standalone_ident: Option<#access_type>,
            ) -> Result<Self::Value, #error_type>
            where
                #error_type: ::xmlity::de::Error,
            {
                #(#body)*
            }
        }))
    }

    fn visit_comment_fn(
        &self,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError> {
        let value_ident = Ident::new("__value", Span::mixed_site());
        let access_type: Type = parse_quote!(__XmlityAccess);
        let error_type: Type = parse_quote!(__XmlityError);

        let body =
            self.visit_comment_fn_body(visitor_lifetime, &value_ident, &access_type, &error_type)?;

        let Some(body) = body else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            fn visit_comment<#error_type, #access_type: AsRef<[u8]>>(self, #value_ident: #access_type) -> Result<Self::Value, #error_type>
            where
                #error_type: ::xmlity::de::Error,
            {
                #(#body)*
            }
        }))
    }

    fn visit_doctype_fn(
        &self,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError> {
        let value_ident = Ident::new("__value", Span::mixed_site());
        let access_type: Type = parse_quote!(__XmlityAccess);
        let error_type: Type = parse_quote!(__XmlityError);

        let body =
            self.visit_doctype_fn_body(visitor_lifetime, &value_ident, &access_type, &error_type)?;

        let Some(body) = body else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            fn visit_doctype<#error_type, #access_type: AsRef<[u8]>>(self, #value_ident: #access_type) -> Result<Self::Value, #error_type>
            where
                #error_type: ::xmlity::de::Error,
            {
                #(#body)*
            }
        }))
    }

    fn visit_none_fn(
        &self,
        visitor_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError> {
        let error_type: Type = parse_quote!(__XmlityError);

        let body = self.visit_none_fn_body(visitor_lifetime, &error_type)?;

        let Some(body) = body else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            fn visit_none<#error_type>(self) -> Result<Self::Value, #error_type>
            where
                #error_type: ::xmlity::de::Error,
            {
                #(#body)*
            }
        }))
    }

    fn visitor_trait_impl(
        &self,
        visitor_ident: &Ident,
        formatter_expecting: &str,
    ) -> Result<ItemImpl, DeriveError> {
        let ident = self.visitor_ident();
        let generics = self.visitor_generics();
        let visitor_lifetime = Lifetime::new("'__visitor", Span::mixed_site());

        let visit_text_fn = self.visit_text_fn(&visitor_lifetime)?;
        let visit_cdata_fn = self.visit_cdata_fn(&visitor_lifetime)?;
        let visit_element_fn = self.visit_element_fn(&visitor_lifetime)?;
        let visit_attribute_fn = self.visit_attribute_fn(&visitor_lifetime)?;
        let visit_seq_fn = self.visit_seq_fn(&visitor_lifetime)?;
        let visit_comment_fn = self.visit_comment_fn(&visitor_lifetime)?;
        let visit_pi_fn = self.visit_pi_fn(&visitor_lifetime)?;
        let visit_decl_fn = self.visit_decl_fn(&visitor_lifetime)?;
        let visit_doctype_fn = self.visit_doctype_fn(&visitor_lifetime)?;
        let visit_none_fn = self.visit_none_fn(&visitor_lifetime)?;
        let value_non_bound_generics = non_bound_generics(generics.deref());

        let mut deserialize_generics = (*generics).to_owned();

        deserialize_generics.params.insert(
            0,
            syn::GenericParam::Lifetime(LifetimeParam::new((visitor_lifetime).clone())),
        );
        let non_bound_deserialize_generics = non_bound_generics(&deserialize_generics);

        Ok(parse_quote! {
            impl #deserialize_generics ::xmlity::de::Visitor<#visitor_lifetime> for #visitor_ident #non_bound_deserialize_generics {
                type Value = #ident #value_non_bound_generics;
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
    fn deserialize_fn_body(
        &self,
        deserializer_ident: &Ident,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Vec<Stmt>, DeriveError>;

    fn ident(&self) -> Cow<'_, Ident>;
    fn generics(&self) -> Cow<'_, Generics>;
}

pub trait DeserializeBuilderExt: DeserializeBuilder {
    fn deserialize_fn(&self, deserialize_lifetime: &Lifetime) -> Result<ImplItemFn, DeriveError>;

    fn deserialize_trait_impl(&self) -> Result<ItemImpl, DeriveError>;
}

impl<T: DeserializeBuilder> DeserializeBuilderExt for T {
    fn deserialize_fn(&self, deserialize_lifetime: &Lifetime) -> Result<ImplItemFn, DeriveError> {
        let deserializer_ident = Ident::new("__deserializer", Span::mixed_site());
        let deserializer_type: Type = parse_quote! { __Deserializer };
        let body = self.deserialize_fn_body(&deserializer_ident, deserialize_lifetime)?;
        Ok(parse_quote! {
                fn deserialize<#deserializer_type>(#deserializer_ident: #deserializer_type) -> Result<Self, <#deserializer_type as ::xmlity::Deserializer<#deserialize_lifetime>>::Error>
                where
                    #deserializer_type: ::xmlity::Deserializer<#deserialize_lifetime>,
                {
                    #(#body)*
                }

        })
    }

    fn deserialize_trait_impl(&self) -> Result<ItemImpl, DeriveError> {
        let ident = self.ident();
        let generics = self.generics();
        let deserialize_lifetime = Lifetime::new("'__deserialize", ident.span());

        let non_bound_generics = non_bound_generics(generics.deref());

        let mut deserialize_generics = (*generics).to_owned();

        deserialize_generics.params.insert(
            0,
            syn::GenericParam::Lifetime(LifetimeParam::new((deserialize_lifetime).clone())),
        );

        let deserialize_fn = self.deserialize_fn(&deserialize_lifetime)?;

        Ok(parse_quote! {
            impl #deserialize_generics ::xmlity::Deserialize<#deserialize_lifetime> for #ident #non_bound_generics  {
                #deserialize_fn
            }
        })
    }
}

pub trait DeserializationGroupBuilderBuilder {
    fn contribute_attributes_fn_body(
        &self,
        attributes_access_ident: &Ident,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Option<Vec<Stmt>>, DeriveError>;

    fn attributes_done_fn_body(
        &self,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Option<Vec<Stmt>>, DeriveError>;

    fn contribute_elements_fn_body(
        &self,
        elements_access_ident: &Ident,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Option<Vec<Stmt>>, DeriveError>;

    fn elements_done_fn_body(
        &self,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Option<Vec<Stmt>>, DeriveError>;

    fn finish_fn_body(&self) -> Result<Vec<Stmt>, DeriveError>;

    fn builder_definition(
        &self,
        builder_ident: &Ident,
        deserialize_lifetime: &Lifetime,
    ) -> Result<ItemStruct, DeriveError>;

    fn builder_constructor(&self, builder_ident: &Ident) -> Result<Vec<Stmt>, DeriveError>;

    fn ident(&self) -> Cow<'_, Ident>;

    fn generics(&self) -> Cow<'_, Generics>;
}

pub trait DeserializationGroupBuilderContentExt: DeserializationGroupBuilderBuilder {
    fn contribute_attributes_fn(
        &self,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError>;

    fn attributes_done_fn(
        &self,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError>;

    fn contribute_elements_fn(
        &self,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError>;

    fn elements_done_fn(
        &self,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError>;

    fn finish_fn(&self) -> Result<ImplItemFn, DeriveError>;

    fn deserialization_group_builder_def(&self) -> Result<ItemStruct, DeriveError>;

    fn deserialization_group_builder_trait_impl(&self) -> Result<ItemImpl, DeriveError>;

    fn deserialization_group_trait_impl(&self) -> Result<ItemImpl, DeriveError>;

    fn total_impl(&self) -> Result<Vec<Item>, DeriveError>;
}

impl<T: DeserializationGroupBuilderBuilder> DeserializationGroupBuilderContentExt for T {
    fn contribute_attributes_fn(
        &self,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError> {
        let attributes_access_ident = syn::Ident::new("__element", proc_macro2::Span::call_site());

        let content =
            self.contribute_attributes_fn_body(&attributes_access_ident, deserialize_lifetime)?;

        let Some(content) = content else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            fn contribute_attributes<A: ::xmlity::de::AttributesAccess<#deserialize_lifetime>>(
                &mut self,
                mut #attributes_access_ident: A,
            ) -> Result<bool, <A as ::xmlity::de::AttributesAccess<#deserialize_lifetime>>::Error> {
                #(#content)*
            }
        }))
    }

    fn attributes_done_fn(
        &self,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError> {
        let content = self.attributes_done_fn_body(deserialize_lifetime)?;

        let Some(content) = content else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            fn attributes_done(&self) -> bool {
                #(#content)*
            }
        }))
    }

    fn contribute_elements_fn(
        &self,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError> {
        let elements_access_ident = syn::Ident::new("__children", proc_macro2::Span::call_site());

        let content =
            self.contribute_elements_fn_body(&elements_access_ident, deserialize_lifetime)?;

        let Some(content) = content else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            fn contribute_elements<A: ::xmlity::de::SeqAccess<#deserialize_lifetime>>(
                &mut self,
              mut #elements_access_ident: A,
            ) -> Result<bool, <A as ::xmlity::de::SeqAccess<#deserialize_lifetime>>::Error> {
                #(#content)*
            }
        }))
    }

    fn elements_done_fn(
        &self,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError> {
        let content = self.elements_done_fn_body(deserialize_lifetime)?;

        let Some(content) = content else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            fn elements_done(&self) -> bool {
                #(#content)*
            }
        }))
    }

    fn finish_fn(&self) -> Result<ImplItemFn, DeriveError> {
        let content = self.finish_fn_body()?;

        Ok(parse_quote! {
        fn finish<E: ::xmlity::de::Error>(self) -> Result<Self::Value, E> {
           #(#content)*
          }
        })
    }

    fn deserialization_group_builder_def(&self) -> Result<ItemStruct, DeriveError> {
        let deserialize_lifetime = Lifetime::new("'__builder", Span::call_site());

        let ident = self.ident();

        let builder_ident = Ident::new(format!("__{ident}Builder").as_str(), ident.span());

        self.builder_definition(&builder_ident, &deserialize_lifetime)
    }

    fn deserialization_group_builder_trait_impl(&self) -> Result<ItemImpl, DeriveError> {
        let deserialize_lifetime = Lifetime::new("'__builder", Span::call_site());

        let ident = self.ident();
        let generics = self.generics();

        let builder_ident = Ident::new(format!("__{ident}Builder").as_str(), ident.span());

        let value_non_bound_generics = non_bound_generics(&generics);

        let mut builder_generics = (*generics).to_owned();

        builder_generics.params.insert(
            0,
            syn::GenericParam::Lifetime(LifetimeParam::new(deserialize_lifetime.clone())),
        );
        let non_bound_builder_generics = non_bound_generics(&builder_generics);

        let contribute_attributes_fn = self.contribute_attributes_fn(&deserialize_lifetime)?;

        let attributes_done_fn = self.attributes_done_fn(&deserialize_lifetime)?;

        let contribute_elements_fn = self.contribute_elements_fn(&deserialize_lifetime)?;

        let elements_done_fn = self.elements_done_fn(&deserialize_lifetime)?;

        let finish_fn = self.finish_fn()?;

        Ok(parse_quote! {
        impl #builder_generics ::xmlity::de::DeserializationGroupBuilder<#deserialize_lifetime> for #builder_ident #non_bound_builder_generics {
          type Value = #ident #value_non_bound_generics;

            #contribute_attributes_fn

            #attributes_done_fn

            #contribute_elements_fn

            #elements_done_fn

            #finish_fn
        }
        })
    }

    fn deserialization_group_trait_impl(&self) -> Result<ItemImpl, DeriveError> {
        let ident = self.ident();
        let generics = self.generics();

        let builder_ident = Ident::new(format!("__{ident}Builder").as_str(), ident.span());

        let deserialize_lifetime = Lifetime::new("'__deserialize", Span::call_site());

        let group_non_bound_generics = non_bound_generics(&generics);

        let mut builder_generics = (*generics).to_owned();

        builder_generics.params.insert(
            0,
            syn::GenericParam::Lifetime(LifetimeParam::new((deserialize_lifetime).clone())),
        );
        let non_bound_builder_generics = non_bound_generics(&builder_generics);

        let builder_constructor = self.builder_constructor(&builder_ident)?;

        Ok(parse_quote! {
            impl #builder_generics ::xmlity::de::DeserializationGroup<#deserialize_lifetime> for #ident #group_non_bound_generics {
                type Builder = #builder_ident #non_bound_builder_generics;

                fn builder() -> Self::Builder {
                    #(#builder_constructor)*
                }
            }
        })
    }

    fn total_impl(&self) -> Result<Vec<Item>, DeriveError> {
        let builder_def = self.deserialization_group_builder_def()?;

        let builder_impl = self.deserialization_group_builder_trait_impl()?;

        let deserialize_impl = self.deserialization_group_trait_impl()?;

        Ok(vec![
            Item::Struct(builder_def),
            Item::Impl(builder_impl),
            Item::Impl(deserialize_impl),
        ])
    }
}
