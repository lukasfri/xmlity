use quote::quote;
use syn::{Ident, Visibility};

use crate::{
    options::{
        GroupOrder, XmlityFieldAttributeDeriveOpts, XmlityFieldElementDeriveOpts,
        XmlityFieldGroupDeriveOpts, XmlityRootGroupDeriveOpts,
    },
    simple_compile_error, DeserializeBuilderField, FieldIdent, XmlityFieldAttributeGroupDeriveOpts,
    XmlityFieldDeriveOpts, XmlityFieldElementGroupDeriveOpts,
};

use super::{all_attributes_done, all_elements_done, constructor_expr, StructType};

fn deserialize_trait_impl<T: quote::ToTokens>(
    ident: &proc_macro2::Ident,
    builder_ident: &proc_macro2::Ident,
    builder_constructor: proc_macro2::TokenStream,
    builder_lifetimes: T,
) -> proc_macro2::TokenStream {
    quote! {
      impl<'de> ::xmlity::de::DeserializationGroup<'de> for #ident {
        type Builder = #builder_ident #builder_lifetimes;

        fn builder() -> Self::Builder {
            #builder_constructor
        }
      }
    }
}

#[allow(clippy::too_many_arguments)]
fn deserialization_group_builder_trait_impl(
    ident: &proc_macro2::Ident,
    builder_ident: &proc_macro2::Ident,
    builder_lifetimes: &proc_macro2::TokenStream,
    element_access_ident: &proc_macro2::Ident,
    contribute_attributes_implementation: proc_macro2::TokenStream,
    attributes_done_implementation: proc_macro2::TokenStream,
    children_access_ident: &proc_macro2::Ident,
    contribute_elements_implementation: proc_macro2::TokenStream,
    elements_done_implementation: proc_macro2::TokenStream,
    finish_implementation: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote! {
    impl<'de> ::xmlity::de::DeserializationGroupBuilder<'de> for #builder_ident #builder_lifetimes {
      type Value = #ident;

      fn contribute_attributes<D: xmlity::de::AttributesAccess<'de>>(
          &mut self,
          mut #element_access_ident: D,
      ) -> Result<bool, <D as xmlity::de::AttributesAccess<'de>>::Error> {
         #contribute_attributes_implementation
      }

      fn attributes_done(&self) -> bool {
          #attributes_done_implementation
      }

      fn contribute_elements<D: xmlity::de::SeqAccess<'de>>(
          &mut self,
        mut #children_access_ident: D,
      ) -> Result<bool, <D as xmlity::de::SeqAccess<'de>>::Error> {
         #contribute_elements_implementation
      }

        fn elements_done(&self) -> bool {
            #elements_done_implementation
        }

      fn finish<E: xmlity::de::Error>(self) -> Result<Self::Value, E> {
        #finish_implementation
      }
    }
    }
}

fn builder_struct_definition_expr<T: quote::ToTokens>(
    ident: T,
    element_fields: impl IntoIterator<
        Item = DeserializeBuilderField<FieldIdent, XmlityFieldElementDeriveOpts>,
    >,
    attribute_fields: impl IntoIterator<
        Item = DeserializeBuilderField<FieldIdent, XmlityFieldAttributeDeriveOpts>,
    >,
    group_fields: impl IntoIterator<Item = DeserializeBuilderField<FieldIdent, XmlityFieldGroupDeriveOpts>>
        + Clone,
    constructor_type: StructType,
    visibility: Visibility,
) -> proc_macro2::TokenStream {
    let local_value_expressions_constructors = attribute_fields
        .into_iter()
        .map(|a| a.map_options(XmlityFieldDeriveOpts::Attribute))
        .chain(
            element_fields
                .into_iter()
                .map(|a| a.map_options(XmlityFieldDeriveOpts::Element)),
        )
        .map(
            |DeserializeBuilderField {
                 builder_field_ident,
                 field_type,
                 ..
             }| {
                let expression = quote! {
                    ::core::option::Option<#field_type>
                };
                (builder_field_ident, expression)
            },
        );
    let group_value_expressions_constructors = group_fields.clone().into_iter().map(
        |DeserializeBuilderField {
             builder_field_ident,
             field_type,
             ..
         }| {
            let expression = quote! {
                <#field_type as ::xmlity::de::DeserializationGroup<'de>>::Builder
            };

            (builder_field_ident, expression)
        },
    );

    let value_expressions_constructors =
        local_value_expressions_constructors.chain(group_value_expressions_constructors);

    super::struct_definition_expr(
        ident,
        // Builder only needs lifetime if there are groups
        if group_fields.into_iter().next().is_some() {
            quote! {<'de>}
        } else {
            quote! {}
        },
        value_expressions_constructors,
        constructor_type,
        visibility,
    )
}

fn finish_constructor_expr<T: quote::ToTokens>(
    ident: T,
    element_fields: impl IntoIterator<
        Item = DeserializeBuilderField<FieldIdent, XmlityFieldElementDeriveOpts>,
    >,
    attribute_fields: impl IntoIterator<
        Item = DeserializeBuilderField<FieldIdent, XmlityFieldAttributeDeriveOpts>,
    >,
    group_fields: impl IntoIterator<
        Item = DeserializeBuilderField<FieldIdent, XmlityFieldGroupDeriveOpts>,
    >,
    constructor_type: StructType,
) -> proc_macro2::TokenStream {
    let local_value_expressions_constructors = attribute_fields.into_iter()
      .map(|a| a.map_options(XmlityFieldDeriveOpts::Attribute))
      .chain(element_fields.into_iter().map(|a| a.map_options(XmlityFieldDeriveOpts::Element)))
      .map(|DeserializeBuilderField { builder_field_ident, field_ident, options, .. }| {
          let expression = if matches!(options, XmlityFieldDeriveOpts::Element(XmlityFieldElementDeriveOpts {default: true}) | XmlityFieldDeriveOpts::Attribute(XmlityFieldAttributeDeriveOpts {default: true})) {
              quote! {
                  ::core::option::Option::unwrap_or_default(self.#builder_field_ident)
              }
          } else {
              quote! {
                  ::core::option::Option::ok_or(self.#builder_field_ident, ::xmlity::de::Error::missing_field(stringify!(#field_ident)))?
              }
          };
          (field_ident, expression)
      });
    let group_value_expressions_constructors = group_fields.into_iter().map(
        |DeserializeBuilderField {
             builder_field_ident,
             field_ident,
             ..
         }| {
            let expression = quote! {
                ::xmlity::de::DeserializationGroupBuilder::finish::<E>(self.#builder_field_ident)?
            };

            (field_ident, expression)
        },
    );

    let value_expressions_constructors =
        local_value_expressions_constructors.chain(group_value_expressions_constructors);

    constructor_expr(ident, value_expressions_constructors, constructor_type)
}

fn builder_constructor_expr<T: quote::ToTokens>(
    ident: T,
    element_fields: impl IntoIterator<
        Item = DeserializeBuilderField<FieldIdent, XmlityFieldElementDeriveOpts>,
    >,
    attribute_fields: impl IntoIterator<
        Item = DeserializeBuilderField<FieldIdent, XmlityFieldAttributeDeriveOpts>,
    >,
    group_fields: impl IntoIterator<
        Item = DeserializeBuilderField<FieldIdent, XmlityFieldGroupDeriveOpts>,
    >,
    constructor_type: StructType,
) -> proc_macro2::TokenStream {
    let local_value_expressions_constructors = attribute_fields
        .into_iter()
        .map(|a| a.map_options(XmlityFieldDeriveOpts::Attribute))
        .chain(
            element_fields
                .into_iter()
                .map(|a| a.map_options(XmlityFieldDeriveOpts::Element)),
        )
        .map(|DeserializeBuilderField { field_ident, .. }| {
            let expression = quote! {
                ::core::option::Option::None
            };
            (field_ident, expression)
        });
    let group_value_expressions_constructors = group_fields.into_iter().map(
        |DeserializeBuilderField {
             field_ident,
             field_type,
             ..
         }| {
            let expression = quote! {
                <#field_type as ::xmlity::de::DeserializationGroup>::builder()
            };

            (field_ident, expression)
        },
    );

    let value_expressions_constructors =
        local_value_expressions_constructors.chain(group_value_expressions_constructors);

    constructor_expr(ident, value_expressions_constructors, constructor_type)
}

pub fn derive_struct_deserialize_fn(
    ident: &Ident,
    builder_ident: &Ident,
    data_struct: &syn::DataStruct,
    opts: XmlityRootGroupDeriveOpts,
    visibility: Visibility,
) -> darling::Result<proc_macro2::TokenStream> {
    let constructor_type = match &data_struct.fields {
        syn::Fields::Named(_) => StructType::Named,
        syn::Fields::Unnamed(_) => StructType::Unnamed,
        _ => unreachable!(),
    };

    let fields = match &data_struct.fields {
        syn::Fields::Named(fields) => fields
            .named
            .iter()
            .map(|f| {
                let field_ident = f.ident.clone().expect("Named struct");

                darling::Result::Ok(DeserializeBuilderField {
                    builder_field_ident: FieldIdent::Named(field_ident.clone()),
                    field_ident: FieldIdent::Named(field_ident),
                    options: XmlityFieldDeriveOpts::from_field(f)?,
                    field_type: f.ty.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?,
        syn::Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, f)| {
                darling::Result::Ok(DeserializeBuilderField {
                    builder_field_ident: FieldIdent::Indexed(syn::Index::from(i)),
                    field_ident: FieldIdent::Indexed(syn::Index::from(i)),
                    options: XmlityFieldDeriveOpts::from_field(f)?,
                    field_type: f.ty.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?,
        _ => unreachable!(),
    };

    let element_fields = fields.clone().into_iter().filter_map(|field| {
        field.map_options_opt(|opt| match opt {
            XmlityFieldDeriveOpts::Element(opts) => Some(opts),
            _ => None,
        })
    });

    let attribute_fields = fields.clone().into_iter().filter_map(|field| {
        field.map_options_opt(|opt| match opt {
            XmlityFieldDeriveOpts::Attribute(opts) => Some(opts),
            _ => None,
        })
    });

    let group_fields = fields.clone().into_iter().filter_map(|field| {
        field.map_options_opt(|opt| match opt {
            XmlityFieldDeriveOpts::Group(opts) => Some(opts),
            _ => None,
        })
    });

    let attribute_group_fields = fields.clone().into_iter().filter_map(|field| {
        field.map_options_opt(|opt| match opt {
            XmlityFieldDeriveOpts::Attribute(opts) => {
                Some(XmlityFieldAttributeGroupDeriveOpts::Attribute(opts))
            }
            XmlityFieldDeriveOpts::Group(opts) => {
                Some(XmlityFieldAttributeGroupDeriveOpts::Group(opts))
            }
            XmlityFieldDeriveOpts::Element(_) => None,
        })
    });

    let element_group_fields = fields.clone().into_iter().filter_map(|field| {
        field.map_options_opt(|opt| match opt {
            XmlityFieldDeriveOpts::Element(opts) => {
                Some(XmlityFieldElementGroupDeriveOpts::Element(opts))
            }
            XmlityFieldDeriveOpts::Group(opts) => {
                Some(XmlityFieldElementGroupDeriveOpts::Group(opts))
            }
            XmlityFieldDeriveOpts::Attribute(_) => None,
        })
    });

    let element_access_ident = syn::Ident::new("__element", proc_macro2::Span::call_site());
    let children_access_ident = syn::Ident::new("__children", proc_macro2::Span::call_site());

    let attribute_visit = super::builder_attribute_field_visitor(
        &element_access_ident,
        quote! {self.},
        attribute_group_fields.clone(),
        quote! {return ::core::result::Result::Ok(false);},
        quote! {return ::core::result::Result::Ok(true);},
        quote! {return ::core::result::Result::Ok(true);},
        match opts.attribute_order {
            GroupOrder::Strict => quote! {},
            GroupOrder::Loose => quote! {return ::core::result::Result::Ok(false);},
            GroupOrder::None => quote! {},
        },
        false,
    );

    let contribute_attributes_implementation = quote! {
        #(#attribute_visit)*

        Ok(false)
    };

    let attributes_done_implementation =
        all_attributes_done(attribute_group_fields, quote! {self.});

    let element_visit = super::builder_element_field_visitor(
        &children_access_ident,
        quote! {self.},
        element_group_fields.clone(),
        quote! {return ::core::result::Result::Ok(false);},
        quote! {return ::core::result::Result::Ok(true);},
        quote! {return ::core::result::Result::Ok(true);},
        match opts.children_order {
            GroupOrder::Strict => quote! {},
            GroupOrder::Loose => quote! {return ::core::result::Result::Ok(false);},
            GroupOrder::None => quote! {},
        },
        match opts.children_order {
            GroupOrder::Strict => true,
            GroupOrder::Loose | GroupOrder::None => false,
        },
    );

    let contribute_elements_implementation = quote! {
        #(#element_visit)*

        Ok(false)
    };

    let elements_done_implementation = all_elements_done(element_group_fields, quote! {self.});

    let finish_constructor = finish_constructor_expr(
        quote! {Self::Value},
        element_fields.clone(),
        attribute_fields.clone(),
        group_fields.clone(),
        constructor_type,
    );

    let finish_implementation = quote! {
      Ok(#finish_constructor)
    };

    let builder_def = builder_struct_definition_expr(
        builder_ident,
        element_fields.clone(),
        attribute_fields.clone(),
        group_fields.clone(),
        constructor_type,
        visibility,
    );

    let builder_lifetimes = if group_fields.clone().next().is_some() {
        quote! {<'de>}
    } else {
        quote! {}
    };

    let builder_impl = deserialization_group_builder_trait_impl(
        ident,
        builder_ident,
        &builder_lifetimes,
        &element_access_ident,
        contribute_attributes_implementation,
        attributes_done_implementation,
        &children_access_ident,
        contribute_elements_implementation,
        elements_done_implementation,
        finish_implementation,
    );

    let builder_constructor = builder_constructor_expr(
        quote! {Self::Builder},
        element_fields,
        attribute_fields,
        group_fields,
        constructor_type,
    );

    let deserialize_impl = deserialize_trait_impl(
        ident,
        builder_ident,
        builder_constructor,
        &builder_lifetimes,
    );

    Ok(quote! {
        #builder_def
        #builder_impl
        #deserialize_impl
    })
}

pub fn derive_deserialization_group_fn(
    ast: syn::DeriveInput,
    opts: XmlityRootGroupDeriveOpts,
) -> darling::Result<proc_macro2::TokenStream> {
    let builder_ident = Ident::new(format!("__{}Builder", ast.ident).as_str(), ast.ident.span());

    match ast.data {
        syn::Data::Struct(data_struct) => {
            derive_struct_deserialize_fn(&ast.ident, &builder_ident, &data_struct, opts, ast.vis)
        }
        syn::Data::Enum(_) => Ok(simple_compile_error("Enums are not supported yet")),
        syn::Data::Union(_) => Ok(simple_compile_error("Unions are not supported yet")),
    }
}
