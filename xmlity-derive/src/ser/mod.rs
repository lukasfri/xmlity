mod attribute;
mod element;
mod group;
pub use attribute::derive_serialize_fn as derive_attribute_serialize_fn;
pub use element::derive_serialize_fn as derive_element_serialize_fn;
pub use group::derive_serialize_fn as derive_group_serialize_fn;

use crate::{
    SerializeField, XmlityFieldAttributeGroupDeriveOpts, XmlityFieldElementGroupDeriveOpts,
};
use quote::{quote, ToTokens};

fn attribute_field_serializer(
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

fn element_field_serializer(
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
