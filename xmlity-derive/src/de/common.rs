use quote::quote;
use syn::{Ident, LifetimeParam};

pub struct VisitorBuilder<'a> {
    ident: &'a syn::Ident,
    generics: &'a syn::Generics,
    visitor_ident: &'a syn::Ident,
    visitor_lifetime: &'a syn::Lifetime,
    formatter_expecting: &'a str,
    visit_text_fn: Option<proc_macro2::TokenStream>,
    visit_cdata_fn: Option<proc_macro2::TokenStream>,
    visit_element_fn: Option<proc_macro2::TokenStream>,
    visit_attribute_fn: Option<proc_macro2::TokenStream>,
    visit_seq_fn: Option<proc_macro2::TokenStream>,
    visit_pi_fn: Option<proc_macro2::TokenStream>,
    visit_decl_fn: Option<proc_macro2::TokenStream>,
    visit_comment_fn: Option<proc_macro2::TokenStream>,
    visit_doctype_fn: Option<proc_macro2::TokenStream>,
    visit_none_fn: Option<proc_macro2::TokenStream>,
}

#[allow(dead_code)]
impl<'a> VisitorBuilder<'a> {
    pub fn new(
        ident: &'a Ident,
        generics: &'a syn::Generics,
        visitor_ident: &'a Ident,
        visitor_lifetime: &'a syn::Lifetime,
        formatter_expecting: &'a str,
    ) -> Self {
        Self {
            ident,
            generics,
            visitor_ident,
            visitor_lifetime,
            formatter_expecting,
            visit_text_fn: None,
            visit_cdata_fn: None,
            visit_element_fn: None,
            visit_attribute_fn: None,
            visit_seq_fn: None,
            visit_pi_fn: None,
            visit_decl_fn: None,
            visit_comment_fn: None,
            visit_doctype_fn: None,
            visit_none_fn: None,
        }
    }

    pub fn visit_text_fn(
        mut self,
        visit_text_fn: impl Into<Option<proc_macro2::TokenStream>>,
    ) -> Self {
        self.visit_text_fn = visit_text_fn.into();
        self
    }
    pub fn visit_cdata_fn(
        mut self,
        visit_cdata_fn: impl Into<Option<proc_macro2::TokenStream>>,
    ) -> Self {
        self.visit_cdata_fn = visit_cdata_fn.into();
        self
    }

    pub fn visit_element_fn(
        mut self,
        visit_element_fn: impl Into<Option<proc_macro2::TokenStream>>,
    ) -> Self {
        self.visit_element_fn = visit_element_fn.into();
        self
    }

    pub fn visit_attribute_fn(
        mut self,
        visit_attribute_fn: impl Into<Option<proc_macro2::TokenStream>>,
    ) -> Self {
        self.visit_attribute_fn = visit_attribute_fn.into();
        self
    }

    pub fn visit_seq_fn(
        mut self,
        visit_seq_fn: impl Into<Option<proc_macro2::TokenStream>>,
    ) -> Self {
        self.visit_seq_fn = visit_seq_fn.into();
        self
    }

    pub fn visit_pi_fn(mut self, visit_pi_fn: impl Into<Option<proc_macro2::TokenStream>>) -> Self {
        self.visit_pi_fn = visit_pi_fn.into();
        self
    }

    pub fn visit_decl_fn(
        mut self,
        visit_decl_fn: impl Into<Option<proc_macro2::TokenStream>>,
    ) -> Self {
        self.visit_decl_fn = visit_decl_fn.into();
        self
    }

    pub fn visit_comment_fn(
        mut self,
        visit_comment_fn: impl Into<Option<proc_macro2::TokenStream>>,
    ) -> Self {
        self.visit_comment_fn = visit_comment_fn.into();
        self
    }

    pub fn visit_doctype_fn(
        mut self,
        visit_doctype_fn: impl Into<Option<proc_macro2::TokenStream>>,
    ) -> Self {
        self.visit_doctype_fn = visit_doctype_fn.into();
        self
    }

    pub fn visit_none_fn(
        mut self,
        visit_none_fn: impl Into<Option<proc_macro2::TokenStream>>,
    ) -> Self {
        self.visit_none_fn = visit_none_fn.into();
        self
    }

    pub fn definition(&self) -> proc_macro2::TokenStream {
        let Self {
            ident,
            generics,
            visitor_ident,
            visitor_lifetime,
            ..
        } = self;
        let non_bound_generics = crate::non_bound_generics(generics);

        let mut deserialize_generics = (*generics).to_owned();

        deserialize_generics.params.insert(
            0,
            syn::GenericParam::Lifetime(LifetimeParam::new((*visitor_lifetime).to_owned())),
        );

        quote! {
            struct #visitor_ident #deserialize_generics {
                marker: ::core::marker::PhantomData<#ident #non_bound_generics>,
                lifetime: ::core::marker::PhantomData<&#visitor_lifetime ()>,
            }
        }
    }

    pub fn trait_impl(&self) -> proc_macro2::TokenStream {
        let Self {
            ident,
            generics,
            visitor_ident,
            visitor_lifetime,
            formatter_expecting,
            visit_text_fn,
            visit_cdata_fn,
            visit_element_fn,
            visit_attribute_fn,
            visit_seq_fn,
            visit_pi_fn,
            visit_decl_fn,
            visit_comment_fn,
            visit_doctype_fn,
            visit_none_fn,
        } = self;
        let non_bound_generics = crate::non_bound_generics(generics);

        let mut deserialize_generics = (*generics).to_owned();

        deserialize_generics.params.insert(
            0,
            syn::GenericParam::Lifetime(LifetimeParam::new((*visitor_lifetime).to_owned())),
        );
        let non_bound_deserialize_generics = crate::non_bound_generics(&deserialize_generics);

        quote! {
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
                #visit_pi_fn
                #visit_decl_fn
                #visit_comment_fn
                #visit_doctype_fn
                #visit_none_fn
            }
        }
    }
}

pub struct DeserializeTraitImplBuilder<'a> {
    ident: &'a syn::Ident,
    generics: &'a syn::Generics,
    deserializer_ident: &'a syn::Ident,
    deserialize_lifetime: &'a syn::Lifetime,
    implementation: proc_macro2::TokenStream,
}

impl<'a> DeserializeTraitImplBuilder<'a> {
    pub fn new(
        ident: &'a syn::Ident,
        generics: &'a syn::Generics,
        deserializer_ident: &'a syn::Ident,
        deserialize_lifetime: &'a syn::Lifetime,
        implementation: proc_macro2::TokenStream,
    ) -> Self {
        Self {
            ident,
            generics,
            deserializer_ident,
            deserialize_lifetime,
            implementation,
        }
    }

    pub fn trait_impl(&self) -> proc_macro2::TokenStream {
        let Self {
            ident,
            generics,
            deserializer_ident,
            deserialize_lifetime,
            implementation,
        } = self;

        let non_bound_generics = crate::non_bound_generics(generics);

        let mut deserialize_generics = (*generics).to_owned();

        deserialize_generics.params.insert(
            0,
            syn::GenericParam::Lifetime(LifetimeParam::new((*deserialize_lifetime).to_owned())),
        );

        quote! {
            impl #deserialize_generics ::xmlity::Deserialize<#deserialize_lifetime> for #ident #non_bound_generics  {
                fn deserialize<D>(#deserializer_ident: D) -> Result<Self, <D as ::xmlity::Deserializer<#deserialize_lifetime>>::Error>
                where
                    D: ::xmlity::Deserializer<#deserialize_lifetime>,
                {
                    #implementation
                }
            }
        }
    }
}
