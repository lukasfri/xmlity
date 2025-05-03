mod builders;
mod serialization_group;
mod serialize;
mod serialize_attribute;
use builders::{SerializeAttributeBuilderExt, SerializeBuilderExt};
use proc_macro2::Span;
pub use serialization_group::DeriveSerializationGroup;
pub use serialize::DeriveSerialize;
use serialize::SingleChildSerializeElementBuilder;
pub use serialize_attribute::DeriveSerializeAttribute;
use serialize_attribute::SimpleSerializeAttributeBuilder;
use syn::{parse_quote, Ident};

use crate::{
    common::FieldIdent,
    options::{
        records::fields::{
            AttributeDeclaredOpts, AttributeDeferredOpts, AttributeOpts, ChildOpts, ElementOpts,
            FieldAttributeGroupOpts, FieldOpts, FieldValueGroupOpts,
        },
        FieldWithOpts, WithExpandedNameExt,
    },
    DeriveError, DeriveResult,
};
use quote::{quote, ToTokens};

fn attribute_group_field_serializer(
    access_ident: impl ToTokens,
    fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, FieldAttributeGroupOpts>>,
) -> DeriveResult<proc_macro2::TokenStream> {
    let fields = fields
      .into_iter()
      .map(|var_field| {
          let field_ident = &var_field.field_ident;

          match var_field.options {
            FieldAttributeGroupOpts::Attribute(AttributeOpts::Declared(opts @ AttributeDeclaredOpts { ..})) => {
                let wrapper_ident = Ident::new("__W", Span::call_site());

                let wrapper = SimpleSerializeAttributeBuilder {
                    ident: &wrapper_ident,
                    generics: &syn::Generics::default(),
                    expanded_name: opts.expanded_name(&field_ident.to_named_ident().to_string()).into_owned(),
                    preferred_prefix: opts.preferred_prefix,
                    enforce_prefix: opts.enforce_prefix,
                    item_type: &var_field.field_type,
                };

                let definition = wrapper.struct_definition();
                let trait_impl = wrapper.serialize_attribute_trait_impl()?;
                let serialize_expr = wrapper.value_expression(&parse_quote!(&self.#field_ident));

                Ok(quote! {{
                    #definition
                    #trait_impl
                    ::xmlity::ser::SerializeAttributes::serialize_attribute(#access_ident, &#serialize_expr)?;
                }})
            },
            FieldAttributeGroupOpts::Attribute(AttributeOpts::Deferred(AttributeDeferredOpts {
                ..
            })) => Ok(quote! {
                ::xmlity::ser::SerializeAttributes::serialize_attribute(#access_ident, &self.#field_ident)?;
            }),
              FieldAttributeGroupOpts::Group(_) => Ok(quote! {
                ::xmlity::ser::SerializationGroup::serialize_attributes(&self.#field_ident, #access_ident)?;
            }),
          }
      }).collect::<DeriveResult<Vec<_>>>()?;

    Ok(quote! {
        #(#fields)*
    })
}

fn element_group_field_serializer(
    access_ident: impl ToTokens,
    fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, FieldValueGroupOpts>>,
) -> DeriveResult<proc_macro2::TokenStream> {
    let fields = fields
    .into_iter()
    .map::<DeriveResult<_>, _>(|var_field| {
        let field_ident = &var_field.field_ident;

        match var_field.options {
            FieldValueGroupOpts::Value(ChildOpts::Element(opts @ ElementOpts {..})) => {
                let wrapper_ident = Ident::new("__W", Span::call_site());

                let wrapper = SingleChildSerializeElementBuilder {
                    ident: &wrapper_ident,
                    required_expanded_name: opts.expanded_name(field_ident.to_named_ident().to_string().as_str())
                    .into_owned(),
                    preferred_prefix: opts.preferred_prefix,
                    enforce_prefix: opts.enforce_prefix,
                    item_type: &var_field.field_type,
                };

                let definition = wrapper.struct_definition();
                let trait_impl = wrapper.serialize_trait_impl()?;
                let serialize_expr = wrapper.value_expression(&parse_quote!(&self.#field_ident));

                Ok(quote! {
                    {
                        #definition
                        #trait_impl
                        ::xmlity::ser::SerializeSeq::serialize_element(#access_ident, &#serialize_expr)?;
                    }
                })
            },
            FieldValueGroupOpts::Value(ChildOpts::Value(_)) => Ok(quote! {
                ::xmlity::ser::SerializeSeq::serialize_element(#access_ident, &self.#field_ident)?;
            }),
            FieldValueGroupOpts::Group(_) => Ok(quote! {
                ::xmlity::ser::SerializationGroup::serialize_children(&self.#field_ident, #access_ident)?;
            }),
        }
    }).collect::<Result<Vec<_>, _>>()?;

    Ok(quote! {
        #(#fields)*
    })
}

fn seq_field_serializer(
    access_ident: impl ToTokens,
    fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, ChildOpts>>,
) -> DeriveResult<proc_macro2::TokenStream> {
    let fields = fields.into_iter().map(|var_field| {
        let field_ident = &var_field.field_ident;

        //TODO: Unify with element_group_field_serializer.
        match var_field.options {
            ChildOpts::Element(opts @ ElementOpts {..}) =>  {
                let wrapper_ident = Ident::new("__W", Span::call_site());

                let wrapper = SingleChildSerializeElementBuilder {
                    ident: &wrapper_ident,
                    required_expanded_name: opts.expanded_name(field_ident.to_named_ident().to_string().as_str())
                    .into_owned(),
                    preferred_prefix: opts.preferred_prefix,
                    enforce_prefix: opts.enforce_prefix,
                    item_type: &var_field.field_type,
                };

                let definition = wrapper.struct_definition();
                let trait_impl = wrapper.serialize_trait_impl()?;
                let serialize_expr = wrapper.value_expression(&parse_quote!(&self.#field_ident));

                DeriveResult::Ok(quote! {
                    {
                        #definition
                        #trait_impl
                        ::xmlity::ser::SerializeSeq::serialize_element(&mut #access_ident, &#serialize_expr)?;
                    }
                })
            },
            ChildOpts::Value(_) => {
                Ok(quote! {
                    ::xmlity::ser::SerializeSeq::serialize_element(&mut #access_ident, &self.#field_ident)?;
                })
            },
        }

    }).collect::<Result<Vec<_>, _>>()?;

    Ok(quote! {
        #(#fields)*
    })
}

fn fields(
    ast: &syn::DeriveInput,
) -> Result<Vec<FieldWithOpts<FieldIdent, FieldOpts>>, DeriveError> {
    let syn::Data::Struct(syn::DataStruct { fields, .. }) = &ast.data else {
        unreachable!()
    };

    match fields {
        syn::Fields::Named(fields) => fields
            .named
            .iter()
            .map(|f| {
                Ok(FieldWithOpts {
                    field_ident: FieldIdent::Named(f.ident.clone().expect("Named struct")),
                    options: FieldOpts::from_field(f)?,
                    field_type: f.ty.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>(),
        syn::Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, f)| {
                Ok(FieldWithOpts {
                    field_ident: FieldIdent::Indexed(syn::Index::from(i)),
                    options: FieldOpts::from_field(f)?,
                    field_type: f.ty.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>(),
        syn::Fields::Unit => Ok(vec![]),
    }
}

fn attribute_group_fields(
    fields: Vec<FieldWithOpts<FieldIdent, FieldOpts>>,
) -> Result<Vec<FieldWithOpts<FieldIdent, FieldAttributeGroupOpts>>, DeriveError> {
    Ok(fields
        .into_iter()
        .filter_map(|field| {
            field.map_options_opt(|opt| match opt {
                FieldOpts::Attribute(opts) => Some(FieldAttributeGroupOpts::Attribute(opts)),
                FieldOpts::Group(opts) => Some(FieldAttributeGroupOpts::Group(opts)),
                FieldOpts::Value(_) => None,
            })
        })
        .collect())
}

fn element_group_fields(
    fields: Vec<FieldWithOpts<FieldIdent, FieldOpts>>,
) -> Result<Vec<FieldWithOpts<FieldIdent, FieldValueGroupOpts>>, DeriveError> {
    Ok(fields
        .into_iter()
        .filter_map(|field| {
            field.map_options_opt(|opt| match opt {
                FieldOpts::Value(opts) => Some(FieldValueGroupOpts::Value(opts)),
                FieldOpts::Group(opts) => Some(FieldValueGroupOpts::Group(opts)),
                FieldOpts::Attribute(_) => None,
            })
        })
        .collect())
}

fn element_fields(
    fields: Vec<FieldWithOpts<FieldIdent, FieldOpts>>,
) -> Result<Vec<FieldWithOpts<FieldIdent, ChildOpts>>, DeriveError> {
    Ok(fields
        .into_iter()
        .filter_map(|field| {
            field.map_options_opt(|opt| match opt {
                FieldOpts::Value(opts) => Some(opts),
                FieldOpts::Group(_) | FieldOpts::Attribute(_) => None,
            })
        })
        .collect())
}
