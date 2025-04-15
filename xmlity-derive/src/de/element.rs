use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{
    parse_quote, spanned::Spanned, Data, DataEnum, DataStruct, DeriveInput, Expr, Field, Ident,
    Lifetime, LifetimeParam, Stmt, Variant,
};

use crate::{
    de::{
        all_attributes_done, all_elements_done, attribute_done, builder_attribute_field_visitor,
        builder_element_field_visitor, constructor_expr, element_done,
    },
    options::{
        ElementOrder, WithExpandedName, WithExpandedNameExt, XmlityRootAttributeDeriveOpts,
        XmlityRootElementDeriveOpts, XmlityRootValueDeriveOpts,
    },
    simple_compile_error, DeriveDeserializeOption, DeriveError, DeriveMacro,
    DeserializeBuilderField, FieldIdent, XmlityFieldAttributeDeriveOpts,
    XmlityFieldAttributeGroupDeriveOpts, XmlityFieldDeriveOpts, XmlityFieldElementDeriveOpts,
    XmlityFieldElementGroupDeriveOpts, XmlityFieldGroupDeriveOpts,
};

use super::{
    common::{DeserializeBuilder, DeserializeBuilderExt, VisitorBuilder, VisitorBuilderExt},
    StructType,
};

pub struct StructElementVisitorBuilder<'a> {
    opts: &'a XmlityRootElementDeriveOpts,
}

impl<'a> StructElementVisitorBuilder<'a> {
    pub fn new(opts: &'a XmlityRootElementDeriveOpts) -> Self {
        Self { opts }
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

        let getter_declarations = visit_element_data_fn_impl_builder_field_decl(
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

        let attribute_loop = if attribute_group_fields.clone().next().is_some() {
            visit_element_data_fn_impl_attribute(
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
            visit_element_data_fn_impl_children(
                element_access_ident,
                element_group_fields,
                allow_unknown_children,
                children_order,
            )
        } else {
            Vec::new()
        };

        let constructor = visit_element_data_fn_impl_constructor(
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
    ) -> Result<Vec<Stmt>, darling::Error> {
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

    fn definition(&self, ast: &syn::DeriveInput) -> Result<syn::ItemStruct, DeriveError> {
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

        let visitor_def = <Self as VisitorBuilder>::definition(self, ast)?;
        let visitor_trait_impl = <Self as VisitorBuilderExt>::visitor_trait_impl(
            self,
            ast,
            &visitor_ident,
            &formatter_expecting,
        )?;

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

pub struct SerializeNoneStructBuilder;

impl SerializeNoneStructBuilder {
    fn new() -> Self {
        Self {}
    }
}

impl VisitorBuilder for SerializeNoneStructBuilder {
    fn visit_seq_fn_body(
        &self,
        ast: &syn::DeriveInput,
        _visitor_lifetime: &Lifetime,
        seq_access_ident: &Ident,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let Data::Struct(DataStruct { fields, .. }) = &ast.data else {
            unreachable!()
        };

        let constructor_type = match &fields {
            syn::Fields::Named(_) => StructType::Named,
            syn::Fields::Unnamed(_) => StructType::Unnamed,
            _ => unreachable!(),
        };

        let fields = crate::de::fields(ast)?.into_iter()
        .map::<(_, Expr), _>(|DeserializeBuilderField { field_ident,  field_type, .. }| {

            (field_ident, parse_quote! {
                ::core::option::Option::ok_or_else(
                    ::xmlity::de::SeqAccess::next_element_seq::<#field_type>(&mut #seq_access_ident)?,
                    ::xmlity::de::Error::missing_data,
                )?
            })
        });

        let constructor = constructor_expr(&ast.ident, fields, &constructor_type);

        Ok(Some(parse_quote! {
            ::core::result::Result::Ok(#constructor)
        }))
    }

    fn definition(&self, ast: &syn::DeriveInput) -> Result<syn::ItemStruct, DeriveError> {
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

impl DeserializeBuilder for SerializeNoneStructBuilder {
    fn deserialize_fn_body(
        &self,
        ast: &syn::DeriveInput,
        deserializer_ident: &Ident,
        _deserialize_lifetime: &Lifetime,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let formatter_expecting = format!("struct {}", ast.ident);

        let visitor_ident = Ident::new("__Visitor", Span::mixed_site());

        let visitor_def = <Self as VisitorBuilder>::definition(self, ast)?;
        let visitor_trait_impl = <Self as VisitorBuilderExt>::visitor_trait_impl(
            self,
            ast,
            &visitor_ident,
            &formatter_expecting,
        )?;

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

pub struct StructAttributeVisitorBuilder<'a> {
    opts: &'a crate::XmlityRootAttributeDeriveOpts,
}

impl<'a> StructAttributeVisitorBuilder<'a> {
    fn new(opts: &'a crate::XmlityRootAttributeDeriveOpts) -> Self {
        Self { opts }
    }
}

impl VisitorBuilder for StructAttributeVisitorBuilder<'_> {
    fn visit_attribute_fn_body(
        &self,
        ast: &syn::DeriveInput,
        visitor_lifetime: &Lifetime,
        attribute_access_ident: &Ident,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let DeriveInput { ident, data, .. } = ast;

        let Data::Struct(data_struct) = data else {
            unreachable!()
        };
        let XmlityRootAttributeDeriveOpts {
            deserialize_any_name,
            ..
        } = self.opts;
        let ident_name = ident.to_string();
        let expanded_name = if *deserialize_any_name {
            None
        } else {
            Some(self.opts.expanded_name(&ident_name))
        };

        let xml_name_identification = expanded_name.map::<Stmt, _>(|qname| {
                parse_quote! {
                    ::xmlity::de::AttributeAccessExt::ensure_name::<<A as ::xmlity::de::AttributeAccess<#visitor_lifetime>>::Error>(&#attribute_access_ident, &#qname)?;
                }
            });

        let deserialization_impl = match &data_struct.fields {
                syn::Fields::Unnamed(fields) if fields.unnamed.len() != 1 => simple_compile_error("Only tuple structs with 1 element are supported"),
                syn::Fields::Unnamed(_) => {
                    parse_quote! {
                        ::core::str::FromStr::from_str(::xmlity::de::AttributeAccess::value(&#attribute_access_ident))
                            .map(#ident)
                            .map_err(::xmlity::de::Error::custom)
                    }
                }
                syn::Fields::Named(_) =>
                    simple_compile_error("Named fields in structs are not supported. Only tuple structs with 1 element are supported"),
                syn::Fields::Unit =>
                    simple_compile_error("Unit structs are not supported. Only tuple structs with 1 element are supported"),
            };

        Ok(Some(parse_quote! {
            #xml_name_identification

            #deserialization_impl
        }))
    }

    fn definition(&self, ast: &syn::DeriveInput) -> Result<syn::ItemStruct, DeriveError> {
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

impl DeserializeBuilder for StructAttributeVisitorBuilder<'_> {
    fn deserialize_fn_body(
        &self,
        ast: &syn::DeriveInput,
        deserializer_ident: &Ident,
        _deserialize_lifetime: &Lifetime,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let formatter_expecting = format!("struct {}", ast.ident);

        let visitor_ident = Ident::new("__Visitor", Span::mixed_site());

        let visitor_def = <Self as VisitorBuilder>::definition(self, ast)?;
        let visitor_trait_impl = <Self as VisitorBuilderExt>::visitor_trait_impl(
            self,
            ast,
            &visitor_ident,
            &formatter_expecting,
        )?;

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

fn visit_element_data_fn_impl_builder_field_decl(
    element_fields: impl IntoIterator<
        Item = DeserializeBuilderField<FieldIdent, XmlityFieldElementDeriveOpts>,
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

fn visit_element_data_fn_impl_constructor(
    ident: &Ident,
    visitor_lifetime: &syn::Lifetime,
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
        .map::<(_, Expr), _>(|DeserializeBuilderField { builder_field_ident, field_ident, options, .. }| {
            let expression = if matches!(options, XmlityFieldDeriveOpts::Element(XmlityFieldElementDeriveOpts {default: true}) | XmlityFieldDeriveOpts::Attribute(XmlityFieldAttributeDeriveOpts {default: true})) {
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

fn visit_element_data_fn_impl_attribute(
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
                let condition = attribute_done(field, quote! {});

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
                let all_some_condition = all_attributes_done(fields, quote! {});

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

struct SeqVisitLoop<
    'a,
    F: IntoIterator<Item = DeserializeBuilderField<FieldIdent, XmlityFieldElementGroupDeriveOpts>>
        + Clone,
> {
    seq_access_ident: &'a Ident,
    allow_unknown_children: bool,
    order: ElementOrder,
    fields: F,
}

impl<
        'a,
        F: IntoIterator<
                Item = DeserializeBuilderField<FieldIdent, XmlityFieldElementGroupDeriveOpts>,
            > + Clone,
    > SeqVisitLoop<'a, F>
{
    fn new(
        seq_access_ident: &'a Ident,
        allow_unknown_children: bool,
        order: ElementOrder,
        fields: F,
    ) -> Self {
        Self {
            seq_access_ident,
            allow_unknown_children,
            order,
            fields,
        }
    }

    fn field_storage(&self) -> proc_macro2::TokenStream {
        quote! {}
    }

    fn access_loop(&self) -> Vec<Stmt> {
        let Self {
            seq_access_ident: access_ident,
            allow_unknown_children,
            order,
            fields,
        } = self;

        let field_visits = builder_element_field_visitor(
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
            ElementOrder::Loose => field_visits.into_iter().zip(fields.clone()).map(|(field_visit, field)| {
                let skip_unknown: Vec<Stmt> = if *allow_unknown_children {
                    let skip_ident = Ident::new("__skip", access_ident.span());
                    parse_quote! {
                        let #skip_ident = ::xmlity::de::SeqAccess::next_element::<::xmlity::types::utils::IgnoredAny>(&mut #access_ident).unwrap_or(None);
                        if ::core::option::Option::is_none(&#skip_ident) {
                            break;
                        }
                        continue;
                    }
                } else {
                    let condition = element_done(field, quote! {});

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
                let skip_unknown: Vec<Stmt> = if *allow_unknown_children {
                    let skip_ident = Ident::new("__skip", access_ident.span());
                    parse_quote! {
                        let #skip_ident = ::xmlity::de::SeqAccess::next_element::<::xmlity::types::utils::IgnoredAny>(&mut #access_ident).unwrap_or(None);
                        if ::core::option::Option::is_none(&#skip_ident) {
                            break
                        }
                    }
                } else {
                    let all_some_condition = all_elements_done(fields.clone(), quote! {});

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
}

fn visit_element_data_fn_impl_children(
    element_access_ident: &Ident,
    fields: impl IntoIterator<Item = DeserializeBuilderField<FieldIdent, XmlityFieldElementGroupDeriveOpts>>
        + Clone,
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

struct EnumNoneVisitorBuilder {}

impl EnumNoneVisitorBuilder {
    fn new() -> Self {
        Self {}
    }
}

impl VisitorBuilder for EnumNoneVisitorBuilder {
    fn visit_seq_fn_body(
        &self,
        DeriveInput { ident, data, .. }: &syn::DeriveInput,
        _visitor_lifetime: &Lifetime,
        seq_access_ident: &Ident,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let data_enum = match &data {
            syn::Data::Enum(data_enum) => data_enum,
            _ => panic!("Wrong options. Only enums can be used for xelement."),
        };
        let variants = data_enum.variants.iter().collect::<Vec<_>>();

        let variants = variants.clone().into_iter().map(|Variant {
            ident: variant_ident,
            fields: variant_fields,
            ..
        }| {
            match variant_fields {
                syn::Fields::Named(_fields) => {
                    None
                }
                syn::Fields::Unnamed(fields) if fields.unnamed.len() != 1 => {
                    None
                }
                syn::Fields::Unnamed(fields) => {
                    let Field {
                        ty: field_type,
                        ..
                    } = fields.unnamed.first().expect("This is guaranteed by the check above");

                    Some(quote! {
                        if let ::core::result::Result::Ok(::core::option::Option::Some(_v)) = ::xmlity::de::SeqAccess::next_element::<#field_type>(&mut #seq_access_ident) {
                            return ::core::result::Result::Ok(#ident::#variant_ident(_v));
                        }
                    })
                }
                syn::Fields::Unit =>{
                    None
                },
            }
        }).collect::<Option<Vec<_>>>().unwrap_or_default();
        let ident_string = ident.to_string();

        Ok(Some(parse_quote! {
            #(#variants)*

            ::core::result::Result::Err(::xmlity::de::Error::no_possible_variant(#ident_string))
        }))
    }

    fn definition(&self, ast: &syn::DeriveInput) -> Result<syn::ItemStruct, DeriveError> {
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

impl DeserializeBuilder for EnumNoneVisitorBuilder {
    fn deserialize_fn_body(
        &self,
        ast: &syn::DeriveInput,
        deserializer_ident: &Ident,
        _deserialize_lifetime: &Lifetime,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let formatter_expecting = format!("enum {}", ast.ident);

        let visitor_ident = Ident::new("__Visitor", Span::mixed_site());

        let visitor_def = <Self as VisitorBuilder>::definition(self, ast)?;
        let visitor_trait_impl = <Self as VisitorBuilderExt>::visitor_trait_impl(
            self,
            ast,
            &visitor_ident,
            &formatter_expecting,
        )?;

        Ok(parse_quote! {
            #visitor_def

            #visitor_trait_impl

            ::xmlity::de::Deserializer::deserialize_seq(#deserializer_ident, #visitor_ident {
                lifetime: ::core::marker::PhantomData,
                marker: ::core::marker::PhantomData,
            })
        })
    }
}

struct EnumValueVisitorBuilder<'a> {
    opts: &'a XmlityRootValueDeriveOpts,
}

impl<'a> EnumValueVisitorBuilder<'a> {
    fn new(opts: &'a XmlityRootValueDeriveOpts) -> Self {
        Self { opts }
    }

    fn str_value_body(
        &self,
        ast: &syn::DeriveInput,
        value_ident: &Ident,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let DeriveInput { ident, data, .. } = ast;
        let syn::Data::Enum(DataEnum { variants, .. }) = data else {
            panic!("This is guaranteed by the caller");
        };

        let variants = variants.clone().into_iter().map(|Variant {
            ident: variant_ident,
            fields: variant_fields,
            ..
        }| {
            let variant_ident_string = self.opts.rename_all.apply_to_variant(&variant_ident.to_string());
            match variant_fields {
                syn::Fields::Named(_fields) => {
                    None
                }
                syn::Fields::Unnamed(_fields) => {
                    None
                }
                syn::Fields::Unit =>{
                    Some(quote! {
                        if ::core::primitive::str::trim(::core::ops::Deref::deref(&#value_ident)) == #variant_ident_string {
                            return ::core::result::Result::Ok(#ident::#variant_ident);
                        }
                    })
                },
            }
        }).collect::<Option<Vec<_>>>();

        let variants = match variants {
            Some(variants) => variants,
            None => return Ok(None),
        };

        let ident_string = ident.to_string();

        Ok(Some(parse_quote! {
            #(#variants)*

            ::core::result::Result::Err(::xmlity::de::Error::no_possible_variant(#ident_string))
        }))
    }
}

impl VisitorBuilder for EnumValueVisitorBuilder<'_> {
    fn visit_text_fn_body(
        &self,
        ast: &syn::DeriveInput,
        _visitor_lifetime: &Lifetime,
        value_ident: &Ident,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let str_ident = Ident::new("__value_str", Span::mixed_site());

        let str_body = self.str_value_body(ast, &str_ident)?;

        let Some(str_body) = str_body else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            let #str_ident = ::xmlity::de::XmlText::as_str(&#value_ident);
            #(#str_body)*
        }))
    }

    fn visit_cdata_fn_body(
        &self,
        ast: &syn::DeriveInput,
        _visitor_lifetime: &Lifetime,
        value_ident: &Ident,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let str_ident = Ident::new("__value_str", Span::mixed_site());

        let str_body = self.str_value_body(ast, &str_ident)?;

        let Some(str_body) = str_body else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            let #str_ident = ::xmlity::de::XmlCData::as_str(&#value_ident);
            #(#str_body)*
        }))
    }

    fn definition(&self, ast: &syn::DeriveInput) -> Result<syn::ItemStruct, DeriveError> {
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

impl DeserializeBuilder for EnumValueVisitorBuilder<'_> {
    fn deserialize_fn_body(
        &self,
        ast: &syn::DeriveInput,
        deserializer_ident: &Ident,
        _deserialize_lifetime: &Lifetime,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let formatter_expecting = format!("enum {}", ast.ident);

        let visitor_ident = Ident::new("__Visitor", Span::mixed_site());

        let visitor_def = <Self as VisitorBuilder>::definition(self, ast)?;
        let visitor_trait_impl = <Self as VisitorBuilderExt>::visitor_trait_impl(
            self,
            ast,
            &visitor_ident,
            &formatter_expecting,
        )?;

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

pub struct DeriveDeserialize;

impl DeriveMacro for DeriveDeserialize {
    fn input_to_derive(ast: &DeriveInput) -> Result<proc_macro2::TokenStream, DeriveError> {
        let opts = DeriveDeserializeOption::parse(ast)?;

        match (&ast.data, &opts) {
            (syn::Data::Struct(_), DeriveDeserializeOption::Element(opts)) => {
                StructElementVisitorBuilder::new(opts)
                    .deserialize_trait_impl(ast)
                    .map(|a| a.to_token_stream())
            }
            (syn::Data::Struct(_), DeriveDeserializeOption::Attribute(opts)) => {
                StructAttributeVisitorBuilder::new(opts)
                    .deserialize_trait_impl(ast)
                    .map(|a| a.to_token_stream())
            }
            (syn::Data::Struct(_), DeriveDeserializeOption::None) => {
                SerializeNoneStructBuilder::new()
                    .deserialize_trait_impl(ast)
                    .map(|a| a.to_token_stream())
            }
            (syn::Data::Struct(_), DeriveDeserializeOption::Value(_opts)) => Ok(
                simple_compile_error("Structs with value options are not supported yet"),
            ),
            (syn::Data::Enum(_), DeriveDeserializeOption::None) => EnumNoneVisitorBuilder::new()
                .deserialize_trait_impl(ast)
                .map(|a| a.to_token_stream()),
            (syn::Data::Enum(_), DeriveDeserializeOption::Value(value_opts)) => {
                EnumValueVisitorBuilder::new(value_opts)
                    .deserialize_trait_impl(ast)
                    .map(|a| a.to_token_stream())
            }
            (syn::Data::Union(_), _) => Ok(simple_compile_error("Unions are not supported yet")),
            _ => Ok(simple_compile_error(
                "Wrong options. Unsupported deserialize.",
            )),
        }
    }
}
