mod element;
mod group;
pub use element::derive_deserialize_fn;
pub use group::derive_deserialization_group_fn;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Ident, Visibility};

use crate::{
    options::{
        XmlityFieldAttributeDeriveOpts, XmlityFieldElementDeriveOpts, XmlityFieldGroupDeriveOpts,
    },
    utils::{self},
    DeserializeBuilderField, FieldIdent, XmlityFieldAttributeGroupDeriveOpts,
    XmlityFieldElementGroupDeriveOpts,
};

struct VisitorBuilder<'a> {
    ident: &'a proc_macro2::Ident,
    visitor_ident: &'a proc_macro2::Ident,
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
    fn new(ident: &'a Ident, visitor_ident: &'a Ident, formatter_expecting: &'a str) -> Self {
        Self {
            ident,
            visitor_ident,
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

    fn visit_text_fn(mut self, visit_text_fn: impl Into<Option<proc_macro2::TokenStream>>) -> Self {
        self.visit_text_fn = visit_text_fn.into();
        self
    }
    fn visit_cdata_fn(
        mut self,
        visit_cdata_fn: impl Into<Option<proc_macro2::TokenStream>>,
    ) -> Self {
        self.visit_cdata_fn = visit_cdata_fn.into();
        self
    }

    fn visit_element_fn(
        mut self,
        visit_element_fn: impl Into<Option<proc_macro2::TokenStream>>,
    ) -> Self {
        self.visit_element_fn = visit_element_fn.into();
        self
    }

    fn visit_attribute_fn(
        mut self,
        visit_attribute_fn: impl Into<Option<proc_macro2::TokenStream>>,
    ) -> Self {
        self.visit_attribute_fn = visit_attribute_fn.into();
        self
    }

    fn visit_seq_fn(mut self, visit_seq_fn: impl Into<Option<proc_macro2::TokenStream>>) -> Self {
        self.visit_seq_fn = visit_seq_fn.into();
        self
    }

    fn visit_pi_fn(mut self, visit_pi_fn: impl Into<Option<proc_macro2::TokenStream>>) -> Self {
        self.visit_pi_fn = visit_pi_fn.into();
        self
    }

    fn visit_decl_fn(mut self, visit_decl_fn: impl Into<Option<proc_macro2::TokenStream>>) -> Self {
        self.visit_decl_fn = visit_decl_fn.into();
        self
    }

    fn visit_comment_fn(
        mut self,
        visit_comment_fn: impl Into<Option<proc_macro2::TokenStream>>,
    ) -> Self {
        self.visit_comment_fn = visit_comment_fn.into();
        self
    }

    fn visit_doctype_fn(
        mut self,
        visit_doctype_fn: impl Into<Option<proc_macro2::TokenStream>>,
    ) -> Self {
        self.visit_doctype_fn = visit_doctype_fn.into();
        self
    }

    fn visit_none_fn(mut self, visit_none_fn: impl Into<Option<proc_macro2::TokenStream>>) -> Self {
        self.visit_none_fn = visit_none_fn.into();
        self
    }

    fn definition(&self) -> proc_macro2::TokenStream {
        let Self {
            ident,
            visitor_ident,
            ..
        } = self;

        quote! {
            struct #visitor_ident<'de> {
                marker: ::core::marker::PhantomData<#ident>,
                lifetime: ::core::marker::PhantomData<&'de ()>,
            }
        }
    }

    fn trait_impl(&self) -> proc_macro2::TokenStream {
        let Self {
            ident,
            visitor_ident,
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

        quote! {
            impl<'de> ::xmlity::de::Visitor<'de> for #visitor_ident<'de> {
                type Value = #ident;
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

#[derive(Clone, Copy)]
enum StructType {
    Named,
    Unnamed,
}

fn named_constructor_expr<I: ToTokens, K: ToTokens, V: ToTokens>(
    ident: I,
    fields: impl IntoIterator<Item = (K, V)>,
) -> proc_macro2::TokenStream {
    let field_tokens = fields.into_iter().map(|(ident, expression)| {
        quote! {
            #ident: #expression,
        }
    });

    quote! {
        #ident {
            #(#field_tokens)*
        }
    }
}

fn unnamed_constructor_expr<I: ToTokens, T: ToTokens>(
    ident: I,
    fields: impl IntoIterator<Item = T>,
) -> proc_macro2::TokenStream {
    let fields = fields.into_iter();

    quote! {
      #ident (
        #(#fields,)*
    )
    }
}

fn constructor_expr<I: ToTokens, T: ToTokens>(
    ident: I,
    fields: impl IntoIterator<Item = (FieldIdent, T)>,
    constructor_type: StructType,
) -> proc_macro2::TokenStream {
    let fields = fields.into_iter();
    match constructor_type {
        StructType::Unnamed => {
            unnamed_constructor_expr(ident, fields.map(|(_, value_expression)| value_expression))
        }
        StructType::Named => named_constructor_expr(
            ident,
            fields.filter_map(|(a, value_expression)| match a {
                FieldIdent::Named(field_ident) => Some((field_ident, value_expression)),
                FieldIdent::Indexed(_) => None,
            }),
        ),
    }
}

fn named_struct_definition_expr<I: ToTokens, K: ToTokens, V: ToTokens>(
    ident: I,
    fields: impl IntoIterator<Item = (K, V)>,
    visibility: Visibility,
) -> proc_macro2::TokenStream {
    let field_tokens = fields.into_iter().map(|(ident, expression)| {
        quote! {
            #ident: #expression,
        }
    });

    quote! {
        #visibility struct #ident {
            #(#field_tokens)*
        }
    }
}

fn unnamed_struct_definition_expr<I: ToTokens, T: ToTokens>(
    ident: I,
    fields: impl IntoIterator<Item = T>,
    visibility: Visibility,
) -> proc_macro2::TokenStream {
    let fields = fields.into_iter();

    quote! {
        #visibility struct #ident (
            #(#fields,)*
        )
    }
}

fn struct_definition_expr<I: ToTokens, T: ToTokens>(
    ident: I,
    fields: impl IntoIterator<Item = (FieldIdent, T)>,
    constructor_type: StructType,
    visibility: Visibility,
) -> proc_macro2::TokenStream {
    let fields = fields.into_iter();
    match constructor_type {
        StructType::Unnamed => unnamed_struct_definition_expr(
            ident,
            fields.map(|(_, value_expression)| value_expression),
            visibility,
        ),
        StructType::Named => named_struct_definition_expr(
            ident,
            fields.filter_map(|(a, value_expression)| match a {
                FieldIdent::Named(field_ident) => Some((field_ident, value_expression)),
                FieldIdent::Indexed(_) => None,
            }),
            visibility,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn builder_attribute_field_visitor<
    F: IntoIterator<Item = DeserializeBuilderField<FieldIdent, XmlityFieldAttributeGroupDeriveOpts>>,
>(
    access_ident: &Ident,
    builder_field_ident_prefix: proc_macro2::TokenStream,
    fields: F,
    if_next_attribute_none: proc_macro2::TokenStream,
    finished_attribute: proc_macro2::TokenStream,
    if_contributed_to_groups: proc_macro2::TokenStream,
    after_attempt: proc_macro2::TokenStream,
    pop_error: bool,
) -> impl Iterator<Item = TokenStream> + use<'_, F> {
    fn attribute_field_deserialize_impl(
        access_ident: &Ident,
        builder_field_ident_prefix: impl ToTokens,
        DeserializeBuilderField {
            builder_field_ident,
            field_type,
            ..
        }: DeserializeBuilderField<FieldIdent, XmlityFieldAttributeDeriveOpts>,
        if_next_attribute_none: proc_macro2::TokenStream,
        finished_attribute: proc_macro2::TokenStream,
        after_attempt: proc_macro2::TokenStream,
        pop_error: bool,
    ) -> proc_macro2::TokenStream {
        let temporary_value_ident = Ident::new("__v", Span::call_site());

        if pop_error {
            quote! {
                if ::core::option::Option::is_none(&#builder_field_ident_prefix #builder_field_ident) {
                    let #temporary_value_ident = ::xmlity::de::AttributesAccess::next_attribute::<#field_type>(&mut #access_ident)?;
                    let ::core::option::Option::Some(#temporary_value_ident) = #temporary_value_ident else {
                        #if_next_attribute_none
                    };
                    #builder_field_ident_prefix #builder_field_ident = ::core::option::Option::Some(#temporary_value_ident);
                    #finished_attribute

                }
            }
        } else {
            quote! {
                if ::core::option::Option::is_none(&#builder_field_ident_prefix #builder_field_ident) {
                    if let ::core::result::Result::Ok(#temporary_value_ident) = ::xmlity::de::AttributesAccess::next_attribute::<#field_type>(&mut #access_ident) {
                        let ::core::option::Option::Some(#temporary_value_ident) = #temporary_value_ident else {
                            #if_next_attribute_none
                        };
                        #builder_field_ident_prefix #builder_field_ident = ::core::option::Option::Some(#temporary_value_ident);
                        #finished_attribute
                    }
                    #after_attempt
                }
            }
        }
    }

    fn group_field_deserialize_impl(
        access_ident: &Ident,
        builder_field_ident_prefix: impl ToTokens,
        DeserializeBuilderField {
            builder_field_ident,
            ..
        }: DeserializeBuilderField<FieldIdent, XmlityFieldGroupDeriveOpts>,
        if_contributed_to_groups: proc_macro2::TokenStream,
        after_attempt: proc_macro2::TokenStream,
        pop_error: bool,
    ) -> proc_macro2::TokenStream {
        let contributed_to_attributes_ident =
            Ident::new("__contributed_to_attributes", Span::call_site());

        if pop_error {
            quote! {
                if !::xmlity::de::DeserializationGroupBuilder::attributes_done(&#builder_field_ident_prefix #builder_field_ident) {
                    let #contributed_to_attributes_ident = ::xmlity::de::DeserializationGroupBuilder::contribute_attributes(&mut #builder_field_ident_prefix #builder_field_ident, ::xmlity::de::AttributesAccess::sub_access(&mut #access_ident)?)?;
                    if #contributed_to_attributes_ident {
                        #if_contributed_to_groups
                    }
                }
            }
        } else {
            quote! {
                if !::xmlity::de::DeserializationGroupBuilder::attributes_done(&#builder_field_ident_prefix #builder_field_ident) {
                    if let ::core::result::Result::Ok(#contributed_to_attributes_ident) = ::xmlity::de::DeserializationGroupBuilder::contribute_attributes(&mut #builder_field_ident_prefix #builder_field_ident, ::xmlity::de::AttributesAccess::sub_access(&mut #access_ident)?) {
                        if #contributed_to_attributes_ident {
                            #if_contributed_to_groups
                        }
                    }
                    #after_attempt
                }
            }
        }
    }

    fields
        .into_iter()
        .zip(utils::repeat_clone((
            builder_field_ident_prefix,
            if_next_attribute_none,
            finished_attribute,
            if_contributed_to_groups,
            after_attempt,
        )))
        .map(
            move |(
                var_field,
                (
                    builder_field_ident_prefix,
                    if_next_attribute_none,
                    finished_attribute,
                    if_contributed_to_groups,
                    after_attempt,
                ),
            )| match &var_field.options {
                XmlityFieldAttributeGroupDeriveOpts::Attribute(_) => {
                    attribute_field_deserialize_impl(
                        access_ident,
                        builder_field_ident_prefix,
                        var_field.map_options(|opts| match opts {
                            XmlityFieldAttributeGroupDeriveOpts::Attribute(opts) => opts,
                            _ => unreachable!(),
                        }),
                        if_next_attribute_none,
                        finished_attribute,
                        after_attempt,
                        pop_error,
                    )
                }
                XmlityFieldAttributeGroupDeriveOpts::Group(_) => group_field_deserialize_impl(
                    access_ident,
                    builder_field_ident_prefix,
                    var_field.map_options(|opts| match opts {
                        XmlityFieldAttributeGroupDeriveOpts::Group(opts) => opts,
                        _ => unreachable!(),
                    }),
                    if_contributed_to_groups,
                    after_attempt,
                    pop_error,
                ),
            },
        )
}

#[allow(clippy::too_many_arguments)]
fn builder_element_field_visitor<
    F: IntoIterator<Item = DeserializeBuilderField<FieldIdent, XmlityFieldElementGroupDeriveOpts>>,
>(
    access_ident: &Ident,
    builder_field_ident_prefix: proc_macro2::TokenStream,
    fields: F,
    if_next_element_none: proc_macro2::TokenStream,
    finished_element: proc_macro2::TokenStream,
    if_contributed_to_groups: proc_macro2::TokenStream,
    after_attempt: proc_macro2::TokenStream,
    pop_error: bool,
) -> impl Iterator<Item = TokenStream> + use<'_, F> {
    fn element_field_deserialize_impl(
        access_ident: &Ident,
        builder_field_ident_prefix: impl ToTokens,
        DeserializeBuilderField {
            builder_field_ident,
            field_type,
            ..
        }: DeserializeBuilderField<FieldIdent, XmlityFieldElementDeriveOpts>,
        if_next_element_none: proc_macro2::TokenStream,
        finished_element: proc_macro2::TokenStream,
        after_attempt: proc_macro2::TokenStream,
        pop_error: bool,
    ) -> proc_macro2::TokenStream {
        let temporary_value_ident = Ident::new("__v", Span::call_site());

        if pop_error {
            quote! {
                if ::core::option::Option::is_none(&#builder_field_ident_prefix #builder_field_ident) {
                    let #temporary_value_ident = ::xmlity::de::SeqAccess::next_element_seq::<#field_type>(&mut #access_ident)?;
                    let ::core::option::Option::Some(#temporary_value_ident) = #temporary_value_ident else {
                        #if_next_element_none
                    };
                    #builder_field_ident_prefix #builder_field_ident = ::core::option::Option::Some(#temporary_value_ident);

                    #finished_element
                }
            }
        } else {
            quote! {
                if ::core::option::Option::is_none(&#builder_field_ident_prefix #builder_field_ident) {
                    if let ::core::result::Result::Ok(#temporary_value_ident) = ::xmlity::de::SeqAccess::next_element_seq::<#field_type>(&mut #access_ident) {
                        let ::core::option::Option::Some(#temporary_value_ident) = #temporary_value_ident else {
                            #if_next_element_none
                        };
                        #builder_field_ident_prefix #builder_field_ident = ::core::option::Option::Some(#temporary_value_ident);

                        #finished_element
                    }
                    #after_attempt
                }
            }
        }
    }

    fn group_field_deserialize_impl(
        access_ident: &Ident,
        builder_field_ident_prefix: impl ToTokens,
        DeserializeBuilderField {
            builder_field_ident,
            ..
        }: DeserializeBuilderField<FieldIdent, XmlityFieldGroupDeriveOpts>,
        if_contributed_to_groups: proc_macro2::TokenStream,
        after_attempt: proc_macro2::TokenStream,
        pop_error: bool,
    ) -> proc_macro2::TokenStream {
        let contributed_to_elements_ident =
            Ident::new("__contributed_to_elements", Span::call_site());

        if pop_error {
            quote! {
                if !::xmlity::de::DeserializationGroupBuilder::elements_done(&#builder_field_ident) {
                    let #contributed_to_elements_ident = ::xmlity::de::DeserializationGroupBuilder::contribute_elements(&mut #builder_field_ident_prefix #builder_field_ident, ::xmlity::de::SeqAccess::sub_access(&mut #access_ident)?)?;
                    if #contributed_to_elements_ident {
                        #if_contributed_to_groups
                    }

                }
            }
        } else {
            quote! {
                if !::xmlity::de::DeserializationGroupBuilder::elements_done(&#builder_field_ident) {
                    if let ::core::result::Result::Ok(#contributed_to_elements_ident) = ::xmlity::de::DeserializationGroupBuilder::contribute_elements(&mut #builder_field_ident_prefix #builder_field_ident, ::xmlity::de::SeqAccess::sub_access(&mut #access_ident)?) {
                        if #contributed_to_elements_ident {
                            #if_contributed_to_groups
                        }
                    }
                    #after_attempt
                }
            }
        }
    }

    fields
        .into_iter()
        .zip(utils::repeat_clone((
            builder_field_ident_prefix,
            if_next_element_none,
            finished_element,
            if_contributed_to_groups,
            after_attempt,
        )))
        .map(
            move |(
                var_field,
                (
                    builder_field_ident_prefix,
                    if_next_element_none,
                    finished_element,
                    if_contributed_to_groups,
                    after_attempt,
                ),
            )| match &var_field.options {
                XmlityFieldElementGroupDeriveOpts::Element(_) => element_field_deserialize_impl(
                    access_ident,
                    builder_field_ident_prefix,
                    var_field.map_options(|opts| match opts {
                        XmlityFieldElementGroupDeriveOpts::Element(opts) => opts,
                        _ => unreachable!(),
                    }),
                    if_next_element_none,
                    finished_element,
                    after_attempt,
                    pop_error,
                ),
                XmlityFieldElementGroupDeriveOpts::Group(_) => group_field_deserialize_impl(
                    access_ident,
                    builder_field_ident_prefix,
                    var_field.map_options(|opts| match opts {
                        XmlityFieldElementGroupDeriveOpts::Group(opts) => opts,
                        _ => unreachable!(),
                    }),
                    if_contributed_to_groups,
                    after_attempt,
                    pop_error,
                ),
            },
        )
}

fn attribute_done(
    field: DeserializeBuilderField<FieldIdent, XmlityFieldAttributeGroupDeriveOpts>,
    builder_field_ident_prefix: impl ToTokens,
) -> proc_macro2::TokenStream {
    let DeserializeBuilderField {
        builder_field_ident,
        options,
        ..
    } = field;
    match options {
        XmlityFieldAttributeGroupDeriveOpts::Attribute(_) => quote! {
            ::core::option::Option::is_some(&#builder_field_ident_prefix #builder_field_ident)
        },
        XmlityFieldAttributeGroupDeriveOpts::Group(_) => quote! {
            ::xmlity::de::DeserializationGroupBuilder::attributes_done(&#builder_field_ident_prefix #builder_field_ident)
        },
    }
}

fn all_attributes_done(
    fields: impl IntoIterator<
            Item = DeserializeBuilderField<FieldIdent, XmlityFieldAttributeGroupDeriveOpts>,
        > + Clone,

    builder_field_ident_prefix: impl ToTokens,
) -> proc_macro2::TokenStream {
    if fields.clone().into_iter().next().is_none() {
        return quote! { true };
    }

    let conditions = fields
        .into_iter()
        .map(|field| attribute_done(field, &builder_field_ident_prefix));

    quote! {
        #(#conditions)&&*
    }
}

fn element_done(
    field: DeserializeBuilderField<FieldIdent, XmlityFieldElementGroupDeriveOpts>,
    builder_field_ident_prefix: impl ToTokens,
) -> proc_macro2::TokenStream {
    let DeserializeBuilderField {
        builder_field_ident,
        options,
        ..
    } = field;
    match options {
        XmlityFieldElementGroupDeriveOpts::Element(_) => quote! {
            ::core::option::Option::is_some(&#builder_field_ident_prefix #builder_field_ident)
        },
        XmlityFieldElementGroupDeriveOpts::Group(_) => quote! {
            ::xmlity::de::DeserializationGroupBuilder::elements_done(&#builder_field_ident_prefix #builder_field_ident)
        },
    }
}

fn all_elements_done(
    fields: impl IntoIterator<Item = DeserializeBuilderField<FieldIdent, XmlityFieldElementGroupDeriveOpts>>
        + Clone,

    builder_field_ident_prefix: impl ToTokens,
) -> proc_macro2::TokenStream {
    if fields.clone().into_iter().next().is_none() {
        return quote! { true };
    }

    let conditions = fields
        .into_iter()
        .map(|field| element_done(field, &builder_field_ident_prefix));

    quote! {
        #(#conditions)&&*
    }
}
