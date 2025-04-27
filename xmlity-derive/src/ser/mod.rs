mod serialization_group;
mod serialize;
mod serialize_attribute;
pub use serialization_group::DeriveSerializationGroup;
pub use serialize::DeriveSerialize;
pub use serialize_attribute::DeriveSerializeAttribute;

use crate::{
    options::structs::fields::{
        FieldAttributeGroupOpts, FieldOpts, FieldValueGroupOpts, ChildOpts,
    },
    DeriveError, FieldIdent, SerializeField,
};
use quote::{quote, ToTokens};

fn attribute_group_field_serializer(
    access_ident: impl ToTokens,
    fields: impl IntoIterator<Item = SerializeField<FieldAttributeGroupOpts>>,
) -> proc_macro2::TokenStream {
    let fields = fields
      .into_iter()
      .map(|var_field| {
          let field_ident = &var_field.field_ident;

          match var_field.options {
            FieldAttributeGroupOpts::Attribute(_) => quote! {
                ::xmlity::ser::SerializeAttributes::serialize_attribute(&mut #access_ident, &self.#field_ident)?;
            },
              FieldAttributeGroupOpts::Group(_) => quote! {
                ::xmlity::ser::SerializationGroup::serialize_attributes(&self.#field_ident, &mut #access_ident)?;
            },
          }
      });

    quote! {
        #(#fields)*
    }
}

fn element_group_field_serializer(
    access_ident: impl ToTokens,
    fields: impl IntoIterator<Item = SerializeField<FieldValueGroupOpts>>,
) -> proc_macro2::TokenStream {
    let fields = fields
    .into_iter()
    .map(|var_field| {
        let field_ident = &var_field.field_ident;

        match var_field.options {
          FieldValueGroupOpts::Value(_) => quote! {
              ::xmlity::ser::SerializeChildren::serialize_child(&mut #access_ident, &self.#field_ident)?;
          },
          FieldValueGroupOpts::Group(_) => quote! {
              ::xmlity::ser::SerializationGroup::serialize_children(&self.#field_ident, &mut #access_ident)?;
          },
        }
    });

    quote! {
        #(#fields)*
    }
}

fn seq_field_serializer(
    access_ident: impl ToTokens,
    fields: impl IntoIterator<Item = SerializeField<ChildOpts>>,
) -> proc_macro2::TokenStream {
    let fields = fields.into_iter().map(|var_field| {
        let field_ident = &var_field.field_ident;

        quote! {
            ::xmlity::ser::SerializeSeq::serialize_element(&mut #access_ident, &self.#field_ident)?;
        }
    });

    quote! {
        #(#fields)*
    }
}

fn fields(ast: &syn::DeriveInput) -> Result<Vec<SerializeField<FieldOpts>>, DeriveError> {
    let syn::Data::Struct(syn::DataStruct { fields, .. }) = &ast.data else {
        unreachable!()
    };

    match fields {
        syn::Fields::Named(fields) => fields
            .named
            .iter()
            .map(|f| {
                Ok(SerializeField {
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
                Ok(SerializeField {
                    field_ident: FieldIdent::Indexed(syn::Index::from(i)),
                    options: FieldOpts::from_field(f)?,
                    field_type: f.ty.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>(),
        syn::Fields::Unit => unreachable!(),
    }
}

fn attribute_group_fields(
    ast: &syn::DeriveInput,
) -> Result<Vec<SerializeField<FieldAttributeGroupOpts>>, DeriveError> {
    Ok(fields(ast)?
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
    ast: &syn::DeriveInput,
) -> Result<Vec<SerializeField<FieldValueGroupOpts>>, DeriveError> {
    Ok(fields(ast)?
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

fn element_fields(ast: &syn::DeriveInput) -> Result<Vec<SerializeField<ChildOpts>>, DeriveError> {
    Ok(fields(ast)?
        .into_iter()
        .filter_map(|field| {
            field.map_options_opt(|opt| match opt {
                FieldOpts::Value(opts) => Some(opts),
                FieldOpts::Group(_) | FieldOpts::Attribute(_) => None,
            })
        })
        .collect())
}
