use std::iter;

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse_quote, DeriveInput, Ident, ImplItemFn, Index, ItemImpl, ItemStruct, Lifetime,
    LifetimeParam,
};

use crate::{
    options::{
        GroupOrder, XmlityFieldAttributeDeriveOpts, XmlityFieldElementDeriveOpts,
        XmlityFieldGroupDeriveOpts, XmlityRootGroupDeriveOpts,
    },
    simple_compile_error, DeriveError, DeriveMacro, DeserializeBuilderField, FieldIdent,
    XmlityFieldAttributeGroupDeriveOpts, XmlityFieldDeriveOpts, XmlityFieldElementGroupDeriveOpts,
};

use super::{all_attributes_done, all_elements_done, constructor_expr, StructType};

trait DeserializationGroupBuilderContent {
    /// Returns the content inside the `DeserializationGroupBuilder::contribute_attributes` function.
    fn contribute_attributes_content(
        &self,
        ast: &syn::DeriveInput,
        attributes_access_ident: &Ident,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Option<proc_macro2::TokenStream>, DeriveError>;

    fn attributes_done_content(
        &self,
        ast: &syn::DeriveInput,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Option<proc_macro2::TokenStream>, DeriveError>;

    fn contribute_elements_content(
        &self,
        ast: &syn::DeriveInput,
        elements_access_ident: &Ident,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Option<proc_macro2::TokenStream>, DeriveError>;

    fn elements_done_content(
        &self,
        ast: &syn::DeriveInput,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Option<proc_macro2::TokenStream>, DeriveError>;

    fn finish_content(
        &self,
        ast: &syn::DeriveInput,
    ) -> Result<proc_macro2::TokenStream, DeriveError>;

    fn builder_definition(
        &self,
        ast: &syn::DeriveInput,
        builder_ident: &Ident,
        deserialize_lifetime: &Lifetime,
    ) -> Result<ItemStruct, DeriveError>;

    fn builder_constructor(
        &self,
        ast: &syn::DeriveInput,
        builder_ident: &Ident,
    ) -> Result<proc_macro2::TokenStream, DeriveError>;
}

trait DeserializationGroupBuilderContentExt: DeserializationGroupBuilderContent {
    fn contribute_attributes_fn(
        &self,
        ast: &syn::DeriveInput,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError>;

    fn attributes_done_fn(
        &self,
        ast: &syn::DeriveInput,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError>;

    fn contribute_elements_fn(
        &self,
        ast: &syn::DeriveInput,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError>;

    fn elements_done_fn(
        &self,
        ast: &syn::DeriveInput,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError>;

    fn finish_fn(&self, ast: &syn::DeriveInput) -> Result<ImplItemFn, DeriveError>;

    fn deserialization_group_builder_def(
        &self,
        ast: &syn::DeriveInput,
    ) -> Result<ItemStruct, DeriveError>;

    fn deserialization_group_builder_impl(
        &self,
        ast: &syn::DeriveInput,
    ) -> Result<ItemImpl, DeriveError>;

    fn deserialize_impl(&self, ast: &syn::DeriveInput) -> Result<TokenStream, DeriveError>;
}

impl<T: DeserializationGroupBuilderContent> DeserializationGroupBuilderContentExt for T {
    fn contribute_attributes_fn(
        &self,
        ast: &syn::DeriveInput,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError> {
        let attributes_access_ident = syn::Ident::new("__element", proc_macro2::Span::call_site());

        let content = self.contribute_attributes_content(
            ast,
            &attributes_access_ident,
            deserialize_lifetime,
        )?;

        let Some(content) = content else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            fn contribute_attributes<A: ::xmlity::de::AttributesAccess<#deserialize_lifetime>>(
                &mut self,
                mut #attributes_access_ident: A,
            ) -> Result<bool, <A as ::xmlity::de::AttributesAccess<#deserialize_lifetime>>::Error> {
                #content
            }
        }))
    }

    fn attributes_done_fn(
        &self,
        ast: &syn::DeriveInput,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError> {
        let content = self.attributes_done_content(ast, deserialize_lifetime)?;

        let Some(content) = content else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            fn attributes_done(&self) -> bool {
                #content
            }
        }))
    }

    fn contribute_elements_fn(
        &self,
        ast: &syn::DeriveInput,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError> {
        let elements_access_ident = syn::Ident::new("__children", proc_macro2::Span::call_site());

        let content =
            self.contribute_elements_content(ast, &elements_access_ident, deserialize_lifetime)?;

        let Some(content) = content else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            fn contribute_elements<A: ::xmlity::de::SeqAccess<#deserialize_lifetime>>(
                &mut self,
              mut #elements_access_ident: A,
            ) -> Result<bool, <A as ::xmlity::de::SeqAccess<#deserialize_lifetime>>::Error> {
                #content
            }
        }))
    }

    fn elements_done_fn(
        &self,
        ast: &syn::DeriveInput,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError> {
        let content = self.elements_done_content(ast, deserialize_lifetime)?;

        let Some(content) = content else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            fn elements_done(&self) -> bool {
                #content
            }
        }))
    }

    fn finish_fn(&self, ast: &syn::DeriveInput) -> Result<ImplItemFn, DeriveError> {
        let finish_content = self.finish_content(ast)?;
        Ok(parse_quote! {
        fn finish<E: ::xmlity::de::Error>(self) -> Result<Self::Value, E> {
           #finish_content
          }
        })
    }

    fn deserialization_group_builder_def(
        &self,
        ast: &syn::DeriveInput,
    ) -> Result<ItemStruct, DeriveError> {
        let deserialize_lifetime = Lifetime::new("'__builder", Span::call_site());

        let builder_ident =
            Ident::new(format!("__{}Builder", ast.ident).as_str(), ast.ident.span());

        self.builder_definition(ast, &builder_ident, &deserialize_lifetime)
    }

    fn deserialization_group_builder_impl(
        &self,
        ast @ DeriveInput {
            ident, generics, ..
        }: &syn::DeriveInput,
    ) -> Result<ItemImpl, DeriveError> {
        let deserialize_lifetime = Lifetime::new("'__builder", Span::call_site());

        let builder_ident =
            Ident::new(format!("__{}Builder", ast.ident).as_str(), ast.ident.span());

        let non_bound_generics = crate::non_bound_generics(generics);

        let mut builder_generics = (*generics).to_owned();

        builder_generics.params.insert(
            0,
            syn::GenericParam::Lifetime(LifetimeParam::new(deserialize_lifetime.clone())),
        );
        let non_bound_builder_generics = crate::non_bound_generics(&builder_generics);

        let contribute_attributes_fn = self.contribute_attributes_fn(ast, &deserialize_lifetime)?;

        let attributes_done_fn = self.attributes_done_fn(ast, &deserialize_lifetime)?;

        let contribute_elements_fn = self.contribute_elements_fn(ast, &deserialize_lifetime)?;

        let elements_done_fn = self.elements_done_fn(ast, &deserialize_lifetime)?;

        let finish_fn = self.finish_fn(ast)?;

        Ok(parse_quote! {
        impl #builder_generics ::xmlity::de::DeserializationGroupBuilder<#deserialize_lifetime> for #builder_ident #non_bound_builder_generics {
          type Value = #ident #non_bound_generics;

            #contribute_attributes_fn

            #attributes_done_fn

            #contribute_elements_fn

            #elements_done_fn

            #finish_fn
        }
        })
    }

    fn deserialize_impl(&self, ast: &syn::DeriveInput) -> Result<TokenStream, DeriveError> {
        let builder_ident =
            Ident::new(format!("__{}Builder", ast.ident).as_str(), ast.ident.span());

        let builder_def = self.deserialization_group_builder_def(ast)?;

        let builder_impl = self.deserialization_group_builder_impl(ast)?;

        let deserialize_lifetime = Lifetime::new("'__deserialize", Span::call_site());

        let deserialize_impl = DeserializeGroupTraitImplBuilder::new(
            &ast.ident,
            &ast.generics,
            &deserialize_lifetime,
            &builder_ident,
            &self.builder_constructor(ast, &builder_ident)?,
        )
        .trait_impl();

        Ok(quote! {
            #builder_def
            #builder_impl
            #deserialize_impl
        })
    }
}

pub struct StructGroup<'a> {
    opts: &'a XmlityRootGroupDeriveOpts,
}

impl<'a> StructGroup<'a> {
    pub fn new(opts: &'a XmlityRootGroupDeriveOpts) -> Self {
        Self { opts }
    }

    pub fn constructor_type(ast: &syn::DeriveInput) -> StructType {
        let data_struct = match ast.data {
            syn::Data::Struct(ref data_struct) => data_struct,
            _ => unreachable!(),
        };
        match &data_struct.fields {
            syn::Fields::Named(_) => StructType::Named,
            syn::Fields::Unnamed(_) => StructType::Unnamed,
            _ => unreachable!(),
        }
    }

    pub fn fields(
        ast: &syn::DeriveInput,
    ) -> Result<
        impl IntoIterator<Item = DeserializeBuilderField<FieldIdent, XmlityFieldDeriveOpts>>,
        DeriveError,
    > {
        let data_struct = match ast.data {
            syn::Data::Struct(ref data_struct) => data_struct,
            _ => unreachable!(),
        };

        Ok(match &data_struct.fields {
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
        })
    }

    fn element_fields(
        ast: &syn::DeriveInput,
    ) -> Result<
        impl IntoIterator<Item = DeserializeBuilderField<FieldIdent, XmlityFieldElementDeriveOpts>>
            + use<'_>,
        DeriveError,
    > {
        Ok(Self::fields(ast)?.into_iter().filter_map(|field| {
            field.map_options_opt(|opt| match opt {
                XmlityFieldDeriveOpts::Element(opts) => Some(opts),
                _ => None,
            })
        }))
    }

    fn attribute_fields(
        ast: &syn::DeriveInput,
    ) -> Result<
        impl IntoIterator<Item = DeserializeBuilderField<FieldIdent, XmlityFieldAttributeDeriveOpts>>
            + use<'_>,
        DeriveError,
    > {
        Ok(Self::fields(ast)?.into_iter().filter_map(|field| {
            field.map_options_opt(|opt| match opt {
                XmlityFieldDeriveOpts::Attribute(opts) => Some(opts),
                _ => None,
            })
        }))
    }

    fn group_fields(
        ast: &syn::DeriveInput,
    ) -> Result<
        impl IntoIterator<Item = DeserializeBuilderField<FieldIdent, XmlityFieldGroupDeriveOpts>>
            + use<'_>,
        DeriveError,
    > {
        Ok(Self::fields(ast)?.into_iter().filter_map(|field| {
            field.clone().map_options_opt(|opt| match opt {
                XmlityFieldDeriveOpts::Group(opts) => Some(opts),
                _ => None,
            })
        }))
    }

    fn attribute_group_fields(
        ast: &syn::DeriveInput,
    ) -> Result<
        impl IntoIterator<
                Item = DeserializeBuilderField<FieldIdent, XmlityFieldAttributeGroupDeriveOpts>,
            > + use<'_>,
        DeriveError,
    > {
        Ok(Self::fields(ast)?.into_iter().filter_map(|field| {
            field.clone().map_options_opt(|opt| match opt {
                XmlityFieldDeriveOpts::Attribute(opts) => {
                    Some(XmlityFieldAttributeGroupDeriveOpts::Attribute(opts))
                }
                XmlityFieldDeriveOpts::Group(opts) => {
                    Some(XmlityFieldAttributeGroupDeriveOpts::Group(opts))
                }
                XmlityFieldDeriveOpts::Element(_) => None,
            })
        }))
    }

    fn element_group_fields(
        ast: &syn::DeriveInput,
    ) -> Result<
        impl IntoIterator<
                Item = DeserializeBuilderField<FieldIdent, XmlityFieldElementGroupDeriveOpts>,
            > + use<'_>,
        DeriveError,
    > {
        Ok(Self::fields(ast)?.into_iter().filter_map(|field| {
            field.clone().map_options_opt(|opt| match opt {
                XmlityFieldDeriveOpts::Element(opts) => {
                    Some(XmlityFieldElementGroupDeriveOpts::Element(opts))
                }
                XmlityFieldDeriveOpts::Group(opts) => {
                    Some(XmlityFieldElementGroupDeriveOpts::Group(opts))
                }
                XmlityFieldDeriveOpts::Attribute(_) => None,
            })
        }))
    }
}

impl DeserializationGroupBuilderContent for StructGroup<'_> {
    fn contribute_attributes_content(
        &self,
        ast: &syn::DeriveInput,
        attributes_access_ident: &Ident,
        _deserialize_lifetime: &Lifetime,
    ) -> Result<Option<proc_macro2::TokenStream>, DeriveError> {
        let attribute_visit = super::builder_attribute_field_visitor(
            attributes_access_ident,
            quote! {self.},
            Self::attribute_group_fields(ast)?,
            quote! {return ::core::result::Result::Ok(false);},
            quote! {return ::core::result::Result::Ok(true);},
            quote! {return ::core::result::Result::Ok(true);},
            match self.opts.attribute_order {
                GroupOrder::Strict => quote! {},
                GroupOrder::Loose => quote! {return ::core::result::Result::Ok(false);},
                GroupOrder::None => quote! {},
            },
            false,
        );

        Ok(Some(quote! {
                #(#attribute_visit)*

                Ok(false)

        }))
    }

    fn attributes_done_content(
        &self,
        ast: &syn::DeriveInput,
        _deserialize_lifetime: &Lifetime,
    ) -> Result<Option<proc_macro2::TokenStream>, DeriveError> {
        Ok(Some(all_attributes_done(
            Self::attribute_group_fields(ast)?,
            quote! {self.},
        )))
    }

    fn contribute_elements_content(
        &self,
        ast: &syn::DeriveInput,
        elements_access_ident: &Ident,
        _deserialize_lifetime: &Lifetime,
    ) -> Result<Option<proc_macro2::TokenStream>, DeriveError> {
        let element_visit = super::builder_element_field_visitor(
            elements_access_ident,
            quote! {self.},
            Self::element_group_fields(ast)?,
            quote! {return ::core::result::Result::Ok(false);},
            quote! {return ::core::result::Result::Ok(true);},
            quote! {return ::core::result::Result::Ok(true);},
            match self.opts.children_order {
                GroupOrder::Strict => quote! {},
                GroupOrder::Loose => quote! {return ::core::result::Result::Ok(false);},
                GroupOrder::None => quote! {},
            },
            match self.opts.children_order {
                GroupOrder::Strict => true,
                GroupOrder::Loose | GroupOrder::None => false,
            },
        );

        Ok(Some(quote! {
            #(#element_visit)*

            ::core::result::Result::Ok(false)
        }))
    }

    fn elements_done_content(
        &self,
        ast: &syn::DeriveInput,
        _deserialize_lifetime: &Lifetime,
    ) -> Result<Option<proc_macro2::TokenStream>, DeriveError> {
        Ok(Some(all_elements_done(
            Self::element_group_fields(ast)?,
            quote! {self.},
        )))
    }

    fn finish_content(
        &self,
        ast: &syn::DeriveInput,
    ) -> Result<proc_macro2::TokenStream, DeriveError> {
        let finish_constructor = finish_constructor_expr(
            quote! {Self::Value},
            Self::element_fields(ast)?,
            Self::attribute_fields(ast)?,
            Self::group_fields(ast)?,
            &Self::constructor_type(ast),
        );

        Ok(quote! {
          ::std::result::Result::Ok(#finish_constructor)
        })
    }

    fn builder_definition(
        &self,
        ast: &syn::DeriveInput,
        builder_ident: &Ident,
        deserialize_lifetime: &Lifetime,
    ) -> Result<ItemStruct, DeriveError> {
        let local_value_expressions_constructors = Self::attribute_fields(ast)?
            .into_iter()
            .map(
                |DeserializeBuilderField {
                     builder_field_ident,
                     field_type,
                     ..
                 }| {
                    let expression = parse_quote! {
                        ::core::option::Option<#field_type>
                    };
                    (builder_field_ident, expression)
                },
            )
            .chain(Self::element_fields(ast)?.into_iter().map(
                |DeserializeBuilderField {
                     builder_field_ident,
                     field_type,
                     ..
                 }| {
                    let expression = parse_quote! {
                        ::core::option::Option<#field_type>
                    };
                    (builder_field_ident, expression)
                },
            ));
        let group_value_expressions_constructors = Self::group_fields(ast)?.into_iter().map(
            |DeserializeBuilderField {
                 builder_field_ident,
                 field_type,
                 ..
             }| {
                let expression = parse_quote! {
                    <#field_type as ::xmlity::de::DeserializationGroup<#deserialize_lifetime>>::Builder
                };

                (builder_field_ident, expression)
            },
        );

        let value_expressions_constructors = local_value_expressions_constructors
            .chain(group_value_expressions_constructors)
            .chain(iter::once((
                match Self::constructor_type(ast) {
                    StructType::Named => {
                        FieldIdent::Named(Ident::new("__marker", Span::call_site()))
                    }
                    StructType::Unnamed => FieldIdent::Indexed(Index::from(0)),
                },
                parse_quote! {
                    ::core::marker::PhantomData<&#deserialize_lifetime ()>
                },
            )));

        let mut generics = ast.generics.clone();
        // if group_fields.clone().into_iter().next().is_some()
        generics.params.insert(
            0,
            syn::GenericParam::Lifetime(LifetimeParam::new((*deserialize_lifetime).to_owned())),
        );

        Ok(syn::parse2(super::struct_definition_expr(
            builder_ident,
            // Builder only needs lifetime if there are groups
            Some(&generics),
            value_expressions_constructors,
            &Self::constructor_type(ast),
            &ast.vis,
        ))?)
    }

    fn builder_constructor(
        &self,
        ast: &syn::DeriveInput,
        builder_ident: &Ident,
    ) -> Result<proc_macro2::TokenStream, DeriveError> {
        let local_value_expressions_constructors = Self::attribute_fields(ast)?
            .into_iter()
            .map(|DeserializeBuilderField { field_ident, .. }| {
                let expression = quote! {
                    ::core::option::Option::None
                };
                (field_ident, expression)
            })
            .chain(Self::element_fields(ast)?.into_iter().map(
                |DeserializeBuilderField { field_ident, .. }| {
                    let expression = quote! {
                        ::core::option::Option::None
                    };
                    (field_ident, expression)
                },
            ));
        let group_value_expressions_constructors = Self::group_fields(ast)?.into_iter().map(
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

        let value_expressions_constructors = local_value_expressions_constructors
            .chain(group_value_expressions_constructors)
            .chain(iter::once((
                match Self::constructor_type(ast) {
                    StructType::Named => {
                        FieldIdent::Named(Ident::new("__marker", Span::call_site()))
                    }
                    StructType::Unnamed => FieldIdent::Indexed(Index::from(0)),
                },
                quote! {
                    ::core::marker::PhantomData
                },
            )));

        Ok(constructor_expr(
            builder_ident,
            value_expressions_constructors,
            &Self::constructor_type(ast),
        ))
    }
}

pub struct DeserializeGroupTraitImplBuilder<'a> {
    ident: &'a proc_macro2::Ident,
    generics: &'a syn::Generics,
    deserialize_lifetime: &'a Lifetime,
    builder_ident: &'a proc_macro2::Ident,
    builder_constructor: &'a proc_macro2::TokenStream,
}

impl<'a> DeserializeGroupTraitImplBuilder<'a> {
    pub fn new(
        ident: &'a proc_macro2::Ident,
        generics: &'a syn::Generics,
        deserialize_lifetime: &'a Lifetime,
        builder_ident: &'a proc_macro2::Ident,
        builder_constructor: &'a proc_macro2::TokenStream,
    ) -> Self {
        Self {
            ident,
            generics,
            deserialize_lifetime,
            builder_ident,
            builder_constructor,
        }
    }

    pub fn trait_impl(&self) -> ItemImpl {
        let Self {
            ident,
            generics,
            deserialize_lifetime,
            builder_ident,
            builder_constructor,
        } = self;

        let non_bound_generics = crate::non_bound_generics(generics);

        let mut builder_generics = (*generics).to_owned();

        builder_generics.params.insert(
            0,
            syn::GenericParam::Lifetime(LifetimeParam::new((*deserialize_lifetime).to_owned())),
        );
        let non_bound_builder_generics = crate::non_bound_generics(&builder_generics);

        parse_quote! {
            impl #builder_generics ::xmlity::de::DeserializationGroup<#deserialize_lifetime> for #ident #non_bound_generics {
                type Builder = #builder_ident #non_bound_builder_generics;

                fn builder() -> Self::Builder {
                    #builder_constructor
                }
            }
        }
    }
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
    constructor_type: &StructType,
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

pub struct DeriveDeserializationGroup;

impl DeriveMacro for DeriveDeserializationGroup {
    fn input_to_derive(ast: &DeriveInput) -> Result<proc_macro2::TokenStream, DeriveError> {
        let opts = XmlityRootGroupDeriveOpts::parse(ast)?.unwrap_or_default();

        match &ast.data {
            syn::Data::Struct(_) => StructGroup::new(&opts).deserialize_impl(ast),
            syn::Data::Enum(_) => {
                Ok(simple_compile_error("Enums are not supported yet").to_token_stream())
            }
            syn::Data::Union(_) => {
                Ok(simple_compile_error("Unions are not supported yet").to_token_stream())
            }
        }
    }
}
