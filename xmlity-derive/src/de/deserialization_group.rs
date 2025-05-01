use std::iter;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse_quote, DeriveInput, Ident, ImplItemFn, Index, ItemImpl, ItemStruct, Lifetime,
    LifetimeParam, Stmt,
};

use crate::{
    common::{non_bound_generics, FieldIdent, StructType},
    options::{
        structs::{
            fields::{AttributeOpts, ChildOpts, GroupOpts},
            roots::RootGroupOpts,
        },
        FieldWithOpts, GroupOrder,
    },
    DeriveError, DeriveMacro, DeriveResult,
};

use super::common::{
    all_attributes_done_expr, attribute_fields, attribute_group_fields,
    builder_attribute_field_visitor, builder_element_field_visitor, element_fields,
    element_group_fields, group_fields,
};

use super::common::{all_elements_done_expr, constructor_expr, struct_definition_expr};

trait DeserializationGroupBuilderBuilder {
    fn contribute_attributes_fn_body(
        &self,
        ast: &syn::DeriveInput,
        attributes_access_ident: &Ident,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Option<Vec<Stmt>>, DeriveError>;

    fn attributes_done_fn_body(
        &self,
        ast: &syn::DeriveInput,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Option<Vec<Stmt>>, DeriveError>;

    fn contribute_elements_fn_body(
        &self,
        ast: &syn::DeriveInput,
        elements_access_ident: &Ident,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Option<Vec<Stmt>>, DeriveError>;

    fn elements_done_fn_body(
        &self,
        ast: &syn::DeriveInput,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Option<Vec<Stmt>>, DeriveError>;

    fn finish_fn_body(&self, ast: &syn::DeriveInput) -> Result<Vec<Stmt>, DeriveError>;

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
    ) -> Result<Vec<Stmt>, DeriveError>;
}

trait DeserializationGroupBuilderContentExt: DeserializationGroupBuilderBuilder {
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

    fn total_impl(&self, ast: &syn::DeriveInput) -> Result<TokenStream, DeriveError>;
}

impl<T: DeserializationGroupBuilderBuilder> DeserializationGroupBuilderContentExt for T {
    fn contribute_attributes_fn(
        &self,
        ast: &syn::DeriveInput,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError> {
        let attributes_access_ident = syn::Ident::new("__element", proc_macro2::Span::call_site());

        let content = self.contribute_attributes_fn_body(
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
                #(#content)*
            }
        }))
    }

    fn attributes_done_fn(
        &self,
        ast: &syn::DeriveInput,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError> {
        let content = self.attributes_done_fn_body(ast, deserialize_lifetime)?;

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
        ast: &syn::DeriveInput,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError> {
        let elements_access_ident = syn::Ident::new("__children", proc_macro2::Span::call_site());

        let content =
            self.contribute_elements_fn_body(ast, &elements_access_ident, deserialize_lifetime)?;

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
        ast: &syn::DeriveInput,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Option<ImplItemFn>, DeriveError> {
        let content = self.elements_done_fn_body(ast, deserialize_lifetime)?;

        let Some(content) = content else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            fn elements_done(&self) -> bool {
                #(#content)*
            }
        }))
    }

    fn finish_fn(&self, ast: &syn::DeriveInput) -> Result<ImplItemFn, DeriveError> {
        let content = self.finish_fn_body(ast)?;

        Ok(parse_quote! {
        fn finish<E: ::xmlity::de::Error>(self) -> Result<Self::Value, E> {
           #(#content)*
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

        let value_non_bound_generics = non_bound_generics(generics);

        let mut builder_generics = (*generics).to_owned();

        builder_generics.params.insert(
            0,
            syn::GenericParam::Lifetime(LifetimeParam::new(deserialize_lifetime.clone())),
        );
        let non_bound_builder_generics = non_bound_generics(&builder_generics);

        let contribute_attributes_fn = self.contribute_attributes_fn(ast, &deserialize_lifetime)?;

        let attributes_done_fn = self.attributes_done_fn(ast, &deserialize_lifetime)?;

        let contribute_elements_fn = self.contribute_elements_fn(ast, &deserialize_lifetime)?;

        let elements_done_fn = self.elements_done_fn(ast, &deserialize_lifetime)?;

        let finish_fn = self.finish_fn(ast)?;

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

    fn deserialize_impl(&self, ast: &syn::DeriveInput) -> Result<TokenStream, DeriveError> {
        let syn::DeriveInput {
            ident, generics, ..
        } = ast;

        let builder_ident =
            Ident::new(format!("__{}Builder", ast.ident).as_str(), ast.ident.span());

        let deserialize_lifetime = Lifetime::new("'__deserialize", Span::call_site());

        let group_non_bound_generics = non_bound_generics(generics);

        let mut builder_generics = (*generics).to_owned();

        builder_generics.params.insert(
            0,
            syn::GenericParam::Lifetime(LifetimeParam::new((deserialize_lifetime).clone())),
        );
        let non_bound_builder_generics = non_bound_generics(&builder_generics);

        let builder_constructor = self.builder_constructor(ast, &builder_ident)?;

        Ok(parse_quote! {
            impl #builder_generics ::xmlity::de::DeserializationGroup<#deserialize_lifetime> for #ident #group_non_bound_generics {
                type Builder = #builder_ident #non_bound_builder_generics;

                fn builder() -> Self::Builder {
                    #(#builder_constructor)*
                }
            }
        })
    }

    fn total_impl(&self, ast: &syn::DeriveInput) -> Result<TokenStream, DeriveError> {
        let builder_def = self.deserialization_group_builder_def(ast)?;

        let builder_impl = self.deserialization_group_builder_impl(ast)?;

        let deserialize_impl = self.deserialize_impl(ast)?;
        Ok(quote! {
            #builder_def
            #builder_impl
            #deserialize_impl
        })
    }
}

pub struct DeriveDeserializationGroupStruct<'a> {
    opts: &'a RootGroupOpts,
}

impl<'a> DeriveDeserializationGroupStruct<'a> {
    pub fn new(opts: &'a RootGroupOpts) -> Self {
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
}

impl DeserializationGroupBuilderBuilder for DeriveDeserializationGroupStruct<'_> {
    fn contribute_attributes_fn_body(
        &self,
        ast: &syn::DeriveInput,
        attributes_access_ident: &Ident,
        _deserialize_lifetime: &Lifetime,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let attribute_visit = builder_attribute_field_visitor(
            attributes_access_ident,
            quote! {self.},
            attribute_group_fields(ast)?,
            parse_quote! {return ::core::result::Result::Ok(false);},
            parse_quote! {return ::core::result::Result::Ok(true);},
            parse_quote! {return ::core::result::Result::Ok(true);},
            match self.opts.attribute_order {
                GroupOrder::Strict => parse_quote! {},
                GroupOrder::Loose => parse_quote! {return ::core::result::Result::Ok(false);},
                GroupOrder::None => parse_quote! {},
            },
            false,
        )?;

        Ok(Some(parse_quote! {
                #(#attribute_visit)*

                Ok(false)

        }))
    }

    fn attributes_done_fn_body(
        &self,
        ast: &syn::DeriveInput,
        _deserialize_lifetime: &Lifetime,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let expr = all_attributes_done_expr(attribute_group_fields(ast)?, quote! {self.});

        Ok(Some(parse_quote!(
            #expr
        )))
    }

    fn contribute_elements_fn_body(
        &self,
        ast: &syn::DeriveInput,
        elements_access_ident: &Ident,
        _deserialize_lifetime: &Lifetime,
    ) -> DeriveResult<Option<Vec<Stmt>>> {
        let element_visit = builder_element_field_visitor(
            elements_access_ident,
            quote! {self.},
            element_group_fields(ast)?,
            parse_quote! {return ::core::result::Result::Ok(false);},
            parse_quote! {return ::core::result::Result::Ok(true);},
            parse_quote! {return ::core::result::Result::Ok(true);},
            match self.opts.children_order {
                GroupOrder::Strict => parse_quote! {},
                GroupOrder::Loose => parse_quote! {return ::core::result::Result::Ok(false);},
                GroupOrder::None => parse_quote! {},
            },
            match self.opts.children_order {
                GroupOrder::Strict => true,
                GroupOrder::Loose | GroupOrder::None => false,
            },
        )?;

        Ok(Some(parse_quote! {
            #(#element_visit)*

            ::core::result::Result::Ok(false)
        }))
    }

    fn elements_done_fn_body(
        &self,
        ast: &syn::DeriveInput,
        _deserialize_lifetime: &Lifetime,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let expr = all_elements_done_expr(element_group_fields(ast)?, quote! {self.});

        Ok(Some(parse_quote!(
            #expr
        )))
    }

    fn finish_fn_body(&self, ast: &syn::DeriveInput) -> Result<Vec<Stmt>, DeriveError> {
        let finish_constructor = finish_constructor_expr(
            quote! {Self::Value},
            element_fields(ast)?,
            attribute_fields(ast)?,
            group_fields(ast)?,
            &Self::constructor_type(ast),
        );

        Ok(parse_quote! {
          ::std::result::Result::Ok(#finish_constructor)
        })
    }

    fn builder_definition(
        &self,
        ast: &syn::DeriveInput,
        builder_ident: &Ident,
        deserialize_lifetime: &Lifetime,
    ) -> Result<ItemStruct, DeriveError> {
        let local_value_expressions_constructors = attribute_fields(ast)?
            .into_iter()
            .map(
                |FieldWithOpts {
                     field_ident,
                     field_type,
                     ..
                 }| {
                    let expression = parse_quote! {
                        ::core::option::Option<#field_type>
                    };
                    (field_ident, expression)
                },
            )
            .chain(element_fields(ast)?.into_iter().map(
                |FieldWithOpts {
                     field_ident,
                     field_type,
                     ..
                 }| {
                    let expression = parse_quote! {
                        ::core::option::Option<#field_type>
                    };
                    (field_ident, expression)
                },
            ));
        let group_value_expressions_constructors = group_fields(ast)?.into_iter().map(
            |FieldWithOpts {
                field_ident,
                 field_type,
                 ..
             }| {
                let expression = parse_quote! {
                    <#field_type as ::xmlity::de::DeserializationGroup<#deserialize_lifetime>>::Builder
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
                    StructType::Unit => unreachable!(),
                },
                parse_quote! {
                    ::core::marker::PhantomData<&#deserialize_lifetime ()>
                },
            )));

        let mut generics = ast.generics.clone();
        generics.params.insert(
            0,
            syn::GenericParam::Lifetime(LifetimeParam::new((*deserialize_lifetime).to_owned())),
        );

        Ok(syn::parse2(struct_definition_expr(
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
    ) -> Result<Vec<Stmt>, DeriveError> {
        let local_value_expressions_constructors = attribute_fields(ast)?
            .into_iter()
            .map(|FieldWithOpts { field_ident, .. }| {
                let expression = quote! {
                    ::core::option::Option::None
                };
                (field_ident, expression)
            })
            .chain(element_fields(ast)?.into_iter().map(
                |FieldWithOpts { field_ident, .. }| {
                    let expression = quote! {
                        ::core::option::Option::None
                    };
                    (field_ident, expression)
                },
            ));
        let group_value_expressions_constructors = group_fields(ast)?.into_iter().map(
            |FieldWithOpts {
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
                    StructType::Unit => unreachable!(),
                },
                quote! {
                    ::core::marker::PhantomData
                },
            )));

        let expr = constructor_expr(
            builder_ident,
            value_expressions_constructors,
            &Self::constructor_type(ast),
        );

        Ok(parse_quote!(#expr))
    }
}

fn finish_constructor_expr<T: quote::ToTokens>(
    ident: T,
    element_fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, ChildOpts>>,
    attribute_fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, AttributeOpts>>,
    group_fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, GroupOpts>>,
    constructor_type: &StructType,
) -> proc_macro2::TokenStream {
    let local_value_expressions_constructors = attribute_fields.into_iter()
        .map(|a: FieldWithOpts<FieldIdent, AttributeOpts>| (
            a.field_ident,
            a.options.should_unwrap_default()
        ))
        .chain(element_fields.into_iter().map(|a: FieldWithOpts<FieldIdent, ChildOpts>| (
            a.field_ident,
            a.options.should_unwrap_default()
        )))
      .map(|( field_ident, should_unwrap_default )| {
          let expression = if should_unwrap_default {
              quote! {
                  ::core::option::Option::unwrap_or_default(self.#field_ident)
              }
          } else {
              quote! {
                  ::core::option::Option::ok_or(self.#field_ident, ::xmlity::de::Error::missing_field(stringify!(#field_ident)))?
              }
          };
          (field_ident, expression)
      });
    let group_value_expressions_constructors =
        group_fields
            .into_iter()
            .map(|FieldWithOpts { field_ident, .. }| {
                let expression = quote! {
                    ::xmlity::de::DeserializationGroupBuilder::finish::<E>(self.#field_ident)?
                };

                (field_ident, expression)
            });

    let value_expressions_constructors =
        local_value_expressions_constructors.chain(group_value_expressions_constructors);

    constructor_expr(ident, value_expressions_constructors, constructor_type)
}

enum DeserializationGroupOption {
    Group(RootGroupOpts),
}

impl DeserializationGroupOption {
    pub fn parse(ast: &DeriveInput) -> Result<Self, DeriveError> {
        let group_opts = RootGroupOpts::parse(ast)?.unwrap_or_default();

        Ok(DeserializationGroupOption::Group(group_opts))
    }
}

pub struct DeriveDeserializationGroup;

impl DeriveMacro for DeriveDeserializationGroup {
    fn input_to_derive(ast: &DeriveInput) -> Result<proc_macro2::TokenStream, DeriveError> {
        let DeserializationGroupOption::Group(opts) = DeserializationGroupOption::parse(ast)?;

        match &ast.data {
            syn::Data::Struct(_) => DeriveDeserializationGroupStruct::new(&opts).total_impl(ast),
            syn::Data::Enum(_) => Err(DeriveError::custom(
                "Enums are not supported for deserialization groups.",
            )),
            syn::Data::Union(_) => Err(DeriveError::custom(
                "Unions are not supported for deserialization groups.",
            )),
        }
    }
}
