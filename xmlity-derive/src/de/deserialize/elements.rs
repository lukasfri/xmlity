use proc_macro2::Span;
use quote::quote;
use syn::{parse_quote, spanned::Spanned, DeriveInput, Expr, Ident, Lifetime, LifetimeParam, Stmt};

use crate::{
    de::{
        all_attributes_done_expr, attribute_done_expr, builder_attribute_field_visitor, common::{DeserializeBuilder, VisitorBuilder, VisitorBuilderExt}, constructor_expr, common::SeqVisitLoop, StructType
    },
    options::{ElementOrder, WithExpandedNameExt, XmlityFieldAttributeDeriveOpts, XmlityFieldGroupDeriveOpts, XmlityFieldValueDeriveOpts, XmlityRootElementDeriveOpts},
    DeriveError, DeriveResult, DeserializeBuilderField, FieldIdent,
    XmlityFieldAttributeGroupDeriveOpts, XmlityFieldDeriveOpts, XmlityFieldValueGroupDeriveOpts,
};

pub struct StructElementVisitorBuilder<'a> {
    opts: &'a XmlityRootElementDeriveOpts,
}

impl<'a> StructElementVisitorBuilder<'a> {
    pub fn new(opts: &'a XmlityRootElementDeriveOpts) -> Self {
        Self { opts }
    }
    pub fn field_decl(
        element_fields: impl IntoIterator<
            Item = DeserializeBuilderField<FieldIdent, XmlityFieldValueDeriveOpts>,
        >,
        attribute_fields: impl IntoIterator<
            Item = DeserializeBuilderField<FieldIdent, XmlityFieldAttributeDeriveOpts>,
        >,
        group_fields: impl IntoIterator<
            Item = DeserializeBuilderField<FieldIdent, XmlityFieldGroupDeriveOpts>,
        >,
    ) -> Vec<Stmt> {
        let getter_declarations = attribute_fields
            .into_iter()
            .map::<Stmt, _>(
                |DeserializeBuilderField {
                     builder_field_ident,
                     field_type,
                     ..
                 }| {
                    parse_quote! {
                        let mut #builder_field_ident = ::core::option::Option::<#field_type>::None;
                    }
                },
            )
            .chain(element_fields.into_iter().map::<Stmt, _>(
                |DeserializeBuilderField {
                     builder_field_ident,
                     field_type,
                     ..
                 }| {
                    parse_quote! {
                        let mut #builder_field_ident = ::core::option::Option::<#field_type>::None;
                    }
                },
            )).chain(group_fields.into_iter().map::<Stmt, _>(
                |DeserializeBuilderField {
                     builder_field_ident,
                     field_type,
                     ..
                 }| {
                    parse_quote! {
                        let mut #builder_field_ident = <#field_type as ::xmlity::de::DeserializationGroup>::builder();
                    }
                },
            ));
    
        parse_quote! {
            #(#getter_declarations)*
        }
    }
    
    pub fn constructor_expr(
        ident: &Ident,
        visitor_lifetime: &syn::Lifetime,
        element_fields: impl IntoIterator<
            Item = DeserializeBuilderField<FieldIdent, XmlityFieldValueDeriveOpts>,
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
            .chain(element_fields.into_iter().map(|a| a.map_options(XmlityFieldDeriveOpts::Value)))
            .map::<(_, Expr), _>(|DeserializeBuilderField { builder_field_ident, field_ident, options, .. }| {
                let expression = if matches!(options, XmlityFieldDeriveOpts::Value(XmlityFieldValueDeriveOpts {default: true, ..}) | XmlityFieldDeriveOpts::Attribute(XmlityFieldAttributeDeriveOpts {default: true, ..})) {
                    parse_quote! {
                        ::core::option::Option::unwrap_or_default(#builder_field_ident)
                    }
                } else {
                    parse_quote! {
                        ::core::option::Option::ok_or(#builder_field_ident, ::xmlity::de::Error::missing_field(stringify!(#field_ident)))?
                    }
                };
                (field_ident, expression)
            });
        let group_value_expressions_constructors = group_fields.into_iter().map::<(_, Expr), _>(
            |DeserializeBuilderField {
                 builder_field_ident,
                 field_ident,
                 ..
             }| {
                let expression = parse_quote! {
                    ::xmlity::de::DeserializationGroupBuilder::finish::<<A as ::xmlity::de::AttributesAccess<#visitor_lifetime>>::Error>(#builder_field_ident)?
                };
    
                (field_ident, expression)
            },
        );
    
        let value_expressions_constructors =
            local_value_expressions_constructors.chain(group_value_expressions_constructors);
    
        constructor_expr(ident, value_expressions_constructors, &constructor_type)
    }

    pub fn attribute_access(
        access_ident: &Ident,
        span: proc_macro2::Span,
        fields: impl IntoIterator<
                Item = DeserializeBuilderField<FieldIdent, XmlityFieldAttributeGroupDeriveOpts>,
            > + Clone,
        allow_unknown_attributes: bool,
        order: ElementOrder,
    ) -> Vec<Stmt> {
        let field_visits = builder_attribute_field_visitor(
            access_ident,
            quote! {},
            fields.clone(),
            parse_quote! {break;},
            match order {
                ElementOrder::Loose => parse_quote! {break;},
                ElementOrder::None => parse_quote! {continue;},
            },
            parse_quote! {continue;},
            parse_quote! {},
            match order {
                ElementOrder::Loose => true,
                ElementOrder::None => false,
            },
        );
    
        match order {
            ElementOrder::Loose => field_visits.into_iter().zip(fields).map(|(field_visit, field)| {
                let skip_unknown: Vec<Stmt> = if allow_unknown_attributes {
                    let skip_ident = Ident::new("__skip", span);
                    parse_quote! {
                        let #skip_ident = ::xmlity::de::AttributesAccess::next_attribute::<::xmlity::types::utils::IgnoredAny>(&mut #access_ident).unwrap_or(None);
                        if ::core::option::Option::is_none(&#skip_ident) {
                            break;
                        }
                        continue;
                    }
                } else {
                    let condition = attribute_done_expr(field, quote! {});
    
                    parse_quote! {
                        if #condition {
                            break;
                        } else {
                            return ::core::result::Result::Err(::xmlity::de::Error::unknown_child());
                        }
                    }
                };
    
                parse_quote! {
                    loop {
                        #field_visit
                        #(#skip_unknown)*
                    }
                }
            }).collect(),
            ElementOrder::None => {
                let skip_unknown: Vec<Stmt> = if allow_unknown_attributes {
                    let skip_ident = Ident::new("__skip", span);
                    parse_quote! {
                        let #skip_ident = ::xmlity::de::AttributesAccess::next_attribute::<::xmlity::types::utils::IgnoredAny>(&mut #access_ident).unwrap_or(None);
                        if ::core::option::Option::is_none(&#skip_ident) {
                            break
                        }
                    }
                } else {
                    let all_some_condition = all_attributes_done_expr(fields, quote! {});
    
                    parse_quote! {
                        if #all_some_condition {
                            break;
                        } else {
                            return ::core::result::Result::Err(::xmlity::de::Error::unknown_child());
                        }
                    }
                };
    
                parse_quote! {
                    loop {
                        #(#field_visits)*
                        #(#skip_unknown)*
                    }
                }
            },
        }
    }

    pub fn element_access(
        element_access_ident: &Ident,
        fields: impl IntoIterator<
                Item = DeserializeBuilderField<FieldIdent, XmlityFieldValueGroupDeriveOpts>,
            > + Clone,
        allow_unknown_children: bool,
        order: ElementOrder,
    ) -> Vec<Stmt> {
        let access_ident = Ident::new("__children", element_access_ident.span());

        let visit = SeqVisitLoop::new(&access_ident, allow_unknown_children, order, fields);

        let field_storage = visit.field_storage();
        let access_loop = visit.access_loop();

        parse_quote! {
            let mut #access_ident = ::xmlity::de::ElementAccess::children(#element_access_ident)?;

            #field_storage

            #(#access_loop)*
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn struct_fields_visitor_end<T>(
        struct_ident: &Ident,
        element_access_ident: &Ident,
        visitor_lifetime: &syn::Lifetime,
        fields: T,
        constructor_type: StructType,
        children_order: ElementOrder,
        allow_unknown_children: bool,
        attributes_order: ElementOrder,
        allow_unknown_attributes: bool,
    ) -> Vec<Stmt>
    where
        T: IntoIterator<Item = DeserializeBuilderField<FieldIdent, XmlityFieldDeriveOpts>> + Clone,
        T::IntoIter: Clone,
    {
        let element_fields = fields.clone().into_iter().filter_map(|field| {
            field.map_options_opt(|opt| match opt {
                XmlityFieldDeriveOpts::Value(opts) => Some(opts),
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

        let getter_declarations = Self::field_decl(
            element_fields.clone(),
            attribute_fields.clone(),
            group_fields.clone(),
        );

        let attribute_group_fields = fields.clone().into_iter().filter_map(|field| {
            field.map_options_opt(|opt| match opt {
                XmlityFieldDeriveOpts::Attribute(opts) => {
                    Some(XmlityFieldAttributeGroupDeriveOpts::Attribute(opts))
                }
                XmlityFieldDeriveOpts::Group(opts) => {
                    Some(XmlityFieldAttributeGroupDeriveOpts::Group(opts))
                }
                XmlityFieldDeriveOpts::Value(_) => None,
            })
        });

        let element_group_fields = fields.clone().into_iter().filter_map(|field| {
            field.map_options_opt(|opt| match opt {
                XmlityFieldDeriveOpts::Value(opts) => {
                    Some(XmlityFieldValueGroupDeriveOpts::Value(opts))
                }
                XmlityFieldDeriveOpts::Group(opts) => {
                    Some(XmlityFieldValueGroupDeriveOpts::Group(opts))
                }
                XmlityFieldDeriveOpts::Attribute(_) => None,
            })
        });

        let attribute_loop = if attribute_group_fields.clone().next().is_some() {
            Self::attribute_access(
                element_access_ident,
                struct_ident.span(),
                attribute_group_fields,
                allow_unknown_attributes,
                attributes_order,
            )
        } else {
            Vec::new()
        };

        let children_loop = if element_group_fields.clone().next().is_some() {
            Self::element_access(
                element_access_ident,
                element_group_fields,
                allow_unknown_children,
                children_order,
            )
        } else {
            Vec::new()
        };

        let constructor = Self::constructor_expr(
            struct_ident,
            visitor_lifetime,
            element_fields.clone(),
            attribute_fields.clone(),
            group_fields.clone(),
            constructor_type,
        );

        parse_quote! {
            #(#getter_declarations)*

            #(#attribute_loop)*

            #(#children_loop)*

            ::core::result::Result::Ok(#constructor)
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn visit_element_data_fn_impl(
        ident: &Ident,
        element_access_ident: &Ident,
        visitor_lifetime: &syn::Lifetime,
        fields: &syn::Fields,
        children_order: ElementOrder,
        allow_unknown_children: bool,
        attribute_order: ElementOrder,
        allow_unknown_attributes: bool,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let constructor_type = match fields {
            syn::Fields::Named(_) => StructType::Named,
            syn::Fields::Unnamed(_) => StructType::Unnamed,
            _ => unreachable!(),
        };

        let fields = match fields {
            syn::Fields::Named(fields) => fields
                .named
                .iter()
                .map(|f| {
                    let field_ident = f.ident.clone().expect("Named struct");

                    DeriveResult::Ok(DeserializeBuilderField {
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
                    DeriveResult::Ok(DeserializeBuilderField {
                        builder_field_ident: FieldIdent::Named(Ident::new(
                            &format!("__{}", i),
                            f.span(),
                        )),
                        field_ident: FieldIdent::Indexed(syn::Index::from(i)),
                        options: XmlityFieldDeriveOpts::from_field(f)?,
                        field_type: f.ty.clone(),
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
            _ => unreachable!(),
        };

        Ok(Self::struct_fields_visitor_end(
            ident,
            element_access_ident,
            visitor_lifetime,
            fields,
            constructor_type,
            children_order,
            allow_unknown_children,
            attribute_order,
            allow_unknown_attributes,
        ))
    }

    fn visit_element_unit_fn_impl(ident: &Ident) -> Vec<Stmt> {
        parse_quote! {
            ::core::result::Result::Ok(#ident)
        }
    }
}

impl VisitorBuilder for StructElementVisitorBuilder<'_> {
    fn visit_element_fn_body(
        &self,
        ast: &syn::DeriveInput,
        visitor_lifetime: &Lifetime,
        element_access_ident: &Ident,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let DeriveInput { ident, data, .. } = ast;
        let XmlityRootElementDeriveOpts {
            deserialize_any_name,
            allow_unknown_attributes,
            allow_unknown_children,
            children_order,
            attribute_order,
            ..
        } = self.opts;

        let data_struct = match data {
            syn::Data::Struct(data_struct) => data_struct,
            _ => unreachable!(),
        };

        let ident_name = ident.to_string();
        let expanded_name = self.opts.expanded_name(&ident_name);
        let expanded_name = if *deserialize_any_name {
            None
        } else {
            Some(expanded_name)
        };

        let xml_name_identification = expanded_name.as_ref().map::<Stmt, _>(|qname| {
          parse_quote! {
              ::xmlity::de::ElementAccessExt::ensure_name::<<A as ::xmlity::de::AttributesAccess<#visitor_lifetime>>::Error>(&#element_access_ident, &#qname)?;
          }
      });

        let deserialization_impl = match &data_struct.fields {
            fields @ syn::Fields::Named(_) | fields @ syn::Fields::Unnamed(_) => {
                Self::visit_element_data_fn_impl(
                    ident,
                    element_access_ident,
                    visitor_lifetime,
                    fields,
                    *children_order,
                    *allow_unknown_children,
                    *attribute_order,
                    *allow_unknown_attributes,
                )?
            }
            syn::Fields::Unit => Self::visit_element_unit_fn_impl(ident),
        };

        Ok(Some(parse_quote! {
            #xml_name_identification

            #(#deserialization_impl)*
        }))
    }

    fn visitor_definition(&self, ast: &syn::DeriveInput) -> Result<syn::ItemStruct, DeriveError> {
        let DeriveInput {
            ident, generics, ..
        } = ast;
        let non_bound_generics = crate::non_bound_generics(generics);

        let mut deserialize_generics = (*generics).to_owned();

        let visitor_ident = Ident::new("__Visitor", Span::mixed_site());
        let visitor_lifetime = Lifetime::new("'__visitor", Span::mixed_site());

        deserialize_generics.params.insert(
            0,
            syn::GenericParam::Lifetime(LifetimeParam::new(visitor_lifetime.clone())),
        );

        Ok(parse_quote! {
            struct #visitor_ident #deserialize_generics {
                marker: ::core::marker::PhantomData<#ident #non_bound_generics>,
                lifetime: ::core::marker::PhantomData<&#visitor_lifetime ()>,
            }
        })
    }
}

impl DeserializeBuilder for StructElementVisitorBuilder<'_> {
    fn deserialize_fn_body(
        &self,
        ast: &syn::DeriveInput,
        deserializer_ident: &Ident,
        _deserialize_lifetime: &Lifetime,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let formatter_expecting = format!("struct {}", ast.ident);

        let visitor_ident = Ident::new("__Visitor", Span::mixed_site());

        let visitor_def = self.visitor_definition(ast)?;
        let visitor_trait_impl =
            self.visitor_trait_impl(ast, &visitor_ident, &formatter_expecting)?;

        Ok(parse_quote! {
            #visitor_def

            #visitor_trait_impl

            ::xmlity::de::Deserializer::deserialize_any(#deserializer_ident, #visitor_ident {
                lifetime: ::core::marker::PhantomData,
                marker: ::core::marker::PhantomData,
            })
        })
    }
}
