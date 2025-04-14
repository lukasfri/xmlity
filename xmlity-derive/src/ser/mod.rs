mod attribute;
mod element;
mod group;
pub use attribute::DeriveSerializeAttribute;
pub use element::DeriveSerialize;
pub use group::DeriveSerializationGroup;

use crate::{
    options::XmlityFieldElementDeriveOpts, DeriveError, FieldIdent, SerializeField,
    XmlityFieldAttributeGroupDeriveOpts, XmlityFieldDeriveOpts, XmlityFieldElementGroupDeriveOpts,
};
use quote::{quote, ToTokens};

fn attribute_group_field_serializer(
    access_ident: impl ToTokens,
    fields: impl IntoIterator<Item = SerializeField<XmlityFieldAttributeGroupDeriveOpts>>,
) -> proc_macro2::TokenStream {
    let fields = fields
      .into_iter()
      .map(|var_field| {
          let field_ident = &var_field.field_ident;

          match var_field.options {
            XmlityFieldAttributeGroupDeriveOpts::Attribute(_) => quote! {
                ::xmlity::ser::SerializeAttributes::serialize_attribute(&mut #access_ident, &self.#field_ident)?;
            },
              XmlityFieldAttributeGroupDeriveOpts::Group(_) => quote! {
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
    fields: impl IntoIterator<Item = SerializeField<XmlityFieldElementGroupDeriveOpts>>,
) -> proc_macro2::TokenStream {
    let fields = fields
    .into_iter()
    .map(|var_field| {
        let field_ident = &var_field.field_ident;

        match var_field.options {
          XmlityFieldElementGroupDeriveOpts::Element(_) => quote! {
              ::xmlity::ser::SerializeChildren::serialize_child(&mut #access_ident, &self.#field_ident)?;
          },
          XmlityFieldElementGroupDeriveOpts::Group(_) => quote! {
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
    fields: impl IntoIterator<Item = SerializeField<XmlityFieldElementDeriveOpts>>,
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

fn fields(
    ast: &syn::DeriveInput,
) -> Result<Vec<SerializeField<XmlityFieldDeriveOpts>>, DeriveError> {
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
                    options: XmlityFieldDeriveOpts::from_field(f)?,
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
                    options: XmlityFieldDeriveOpts::from_field(f)?,
                    field_type: f.ty.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>(),
        syn::Fields::Unit => unreachable!(),
    }
}

fn attribute_group_fields(
    ast: &syn::DeriveInput,
) -> Result<Vec<SerializeField<XmlityFieldAttributeGroupDeriveOpts>>, DeriveError> {
    Ok(fields(ast)?
        .into_iter()
        .filter_map(|field| {
            field.map_options_opt(|opt| match opt {
                XmlityFieldDeriveOpts::Attribute(opts) => {
                    Some(XmlityFieldAttributeGroupDeriveOpts::Attribute(opts))
                }
                XmlityFieldDeriveOpts::Group(opts) => {
                    Some(XmlityFieldAttributeGroupDeriveOpts::Group(opts))
                }
                XmlityFieldDeriveOpts::Element(_) => None,
            })
        })
        .collect())
}

fn element_group_fields(
    ast: &syn::DeriveInput,
) -> Result<Vec<SerializeField<XmlityFieldElementGroupDeriveOpts>>, DeriveError> {
    Ok(fields(ast)?
        .into_iter()
        .filter_map(|field| {
            field.map_options_opt(|opt| match opt {
                XmlityFieldDeriveOpts::Element(opts) => {
                    Some(XmlityFieldElementGroupDeriveOpts::Element(opts))
                }
                XmlityFieldDeriveOpts::Group(opts) => {
                    Some(XmlityFieldElementGroupDeriveOpts::Group(opts))
                }
                XmlityFieldDeriveOpts::Attribute(_) => None,
            })
        })
        .collect())
}

fn element_fields(
    ast: &syn::DeriveInput,
) -> Result<Vec<SerializeField<XmlityFieldElementDeriveOpts>>, DeriveError> {
    Ok(fields(ast)?
        .into_iter()
        .filter_map(|field| {
            field.map_options_opt(|opt| match opt {
                XmlityFieldDeriveOpts::Element(opts) => Some(opts),
                XmlityFieldDeriveOpts::Group(_) | XmlityFieldDeriveOpts::Attribute(_) => None,
            })
        })
        .collect())
}
