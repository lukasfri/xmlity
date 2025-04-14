use proc_macro2::Span;
use quote::quote;
use syn::{spanned::Spanned, DataEnum, DataStruct, DeriveInput, Field, Ident, Lifetime, Variant};

use crate::{
    de::{
        all_attributes_done, all_elements_done, attribute_done, builder_attribute_field_visitor,
        builder_element_field_visitor, constructor_expr, element_done,
    }, options::{ElementOrder,  XmlityRootElementDeriveOpts, XmlityRootValueDeriveOpts}, simple_compile_error, DeriveDeserializeOption, DeriveError, DeriveMacro, DeserializeBuilderField, ExpandedName, FieldIdent, XmlityFieldAttributeDeriveOpts, XmlityFieldAttributeGroupDeriveOpts, XmlityFieldDeriveOpts, XmlityFieldElementDeriveOpts, XmlityFieldElementGroupDeriveOpts, XmlityFieldGroupDeriveOpts
};

use super::{common::DeserializeTraitImplBuilder, common::VisitorBuilder, StructType};

trait DeserializeContent {
    /// Returns the content inside the `Deserialize::deserialize` function.
    fn deserialize_content(
        &self,
        ast: &syn::DeriveInput,
        visitor_ident: &Ident,
        visitor_lifetime: &Lifetime,
        deserializer_ident: &Ident,
    ) -> Result<proc_macro2::TokenStream, DeriveError>;
}

trait DeserializeContentExt: DeserializeContent {
    fn deserialize_impl(
        &self,
        ast: &syn::DeriveInput,
    ) -> Result<proc_macro2::TokenStream, DeriveError>;
}

impl<T: DeserializeContent> DeserializeContentExt for T {
    fn deserialize_impl(
        &self,
        ast @ DeriveInput {
            ident,
            generics,
            ..
        }: &syn::DeriveInput,
    ) -> Result<proc_macro2::TokenStream, DeriveError> {
        let visitor_ident =
            Ident::new("__Visitor", ident.span());
        let visitor_lifetime = Lifetime::new("'__visitor", ident.span());
        let deserializer_ident = Ident::new("__deserializer", ident.span());
        let deserialize_lifetime = Lifetime::new("'__deserialize", ident.span());

        let implementation =
            self.deserialize_content(ast, &visitor_ident, &visitor_lifetime, &deserializer_ident)?;

        let trait_impl_builder = DeserializeTraitImplBuilder::new(
            ident,
            generics,
            &deserializer_ident,
            &deserialize_lifetime,
            implementation,
        );

        Ok(trait_impl_builder.trait_impl())
    }
}

pub struct StructElementVisitorBuilder<'a> {
    opts: &'a XmlityRootElementDeriveOpts,
}

impl<'a> StructElementVisitorBuilder<'a> {
    pub fn new(
        opts: &'a XmlityRootElementDeriveOpts,
    ) -> Self {
        Self {
            opts,
            
        }
    }

    fn struct_fields_visitor_end<T>(
        struct_ident: &Ident,
        visitor_lifetime: &syn::Lifetime,
        fields: T,
        constructor_type: StructType,
        children_order: ElementOrder,
        allow_unknown_children: bool,
        attributes_order: ElementOrder,
        allow_unknown_attributes: bool,
    ) -> proc_macro2::TokenStream
    where
        T: IntoIterator<Item = DeserializeBuilderField<FieldIdent, XmlityFieldDeriveOpts>> + Clone,
        T::IntoIter: Clone,
    {
        let element_access_ident = Ident::new("__element", struct_ident.span());
    
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
            Some(visit_element_data_fn_impl_attribute(
                &element_access_ident,
                struct_ident.span(),
                attribute_group_fields,
                allow_unknown_attributes,
                attributes_order,
            ))
        } else {
            None
        };
    
        let children_loop = if element_group_fields.clone().next().is_some() {
            Some(visit_element_data_fn_impl_children(
                &element_access_ident,
                element_group_fields,
                allow_unknown_children,
                children_order,
            ))
        } else {
            None
        };
    
        let constructor = visit_element_data_fn_impl_constructor(
            struct_ident,
            visitor_lifetime,
            element_fields.clone(),
            attribute_fields.clone(),
            group_fields.clone(),
            constructor_type,
        );
    
        quote! {
            #getter_declarations
    
            #attribute_loop
    
            #children_loop
    
            ::core::result::Result::Ok(#constructor)
        }
    }

    fn visit_element_data_fn_impl(
        ident: &Ident,
        visitor_lifetime: &syn::Lifetime,
        fields: &syn::Fields,
        children_order: ElementOrder,
        allow_unknown_children: bool,
        attributes_order: ElementOrder,
        allow_unknown_attributes: bool,
    ) -> Result<proc_macro2::TokenStream, darling::Error> {
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
            visitor_lifetime,
            fields,
            constructor_type,
            children_order,
            allow_unknown_children,
            attributes_order,
            allow_unknown_attributes,
        ))
    }

    fn visit_element_unit_fn_impl(ident: &Ident) -> proc_macro2::TokenStream {
        quote! {
            ::core::result::Result::Ok(#ident)
        }
    }

    fn element_visit_fn(
        ident: &Ident,
        visitor_lifetime: &syn::Lifetime,
        expanded_name: Option<ExpandedName>,
        data_struct: &DataStruct,
        children_order: ElementOrder,
        allow_unknown_children: bool,
        attributes_order: ElementOrder,
        allow_unknown_attributes: bool,
    ) -> Result<proc_macro2::TokenStream, darling::Error> {
        let element_access_ident = Ident::new("__element", ident.span());
        let xml_name_identification = expanded_name.as_ref().map(|qname| {
            quote! {
                ::xmlity::de::ElementAccessExt::ensure_name::<<A as ::xmlity::de::AttributesAccess<#visitor_lifetime>>::Error>(&#element_access_ident, &#qname)?;
            }
        });

        let deserialization_impl = match &data_struct.fields {
            fields @ syn::Fields::Named(_) | fields @ syn::Fields::Unnamed(_) => {
                Self::visit_element_data_fn_impl(
                    ident,
                    visitor_lifetime,
                    fields,
                    children_order,
                    allow_unknown_children,
                    attributes_order,
                    allow_unknown_attributes,
                )?
            }
            syn::Fields::Unit => Self::visit_element_unit_fn_impl(ident),
        };

        Ok(VisitorBuilder::visit_element_fn_signature(
            quote! {
                #xml_name_identification

                #deserialization_impl
            },
            &element_access_ident,
            visitor_lifetime,
        ))
    }
}

impl DeserializeContent for StructElementVisitorBuilder<'_> {
    fn deserialize_content(
        &self,
        ast @ DeriveInput { ident, generics, data, .. }: &syn::DeriveInput,
        visitor_ident: &Ident,
        visitor_lifetime: &Lifetime,
        deserializer_ident: &Ident,
    ) -> Result<proc_macro2::TokenStream, DeriveError> {
        let Self {
            opts,
        } = self;
        let formatter_expecting = format!("struct {}", ident);
        let data_struct = match data {
            syn::Data::Struct(data_struct) => data_struct,
            _ => {
                return Err(darling::Error::custom(format!(
                    "{} can only be derived for structs",
                    formatter_expecting
                ))
                .with_span(ast))?;
            }
        };

        let XmlityRootElementDeriveOpts {
            name,
            namespace,
            children_order,
            allow_unknown_attributes,
            attribute_order,
            allow_unknown_children,
            deserialize_any_name,
            ..
        } = opts;
        let visit_element_fn = {
            let ident_name = ident.to_string();
            let expanded_name = ExpandedName::new(
                name.0.as_ref().unwrap_or(&ident_name),
                namespace.0.as_deref(),
            );
            let expanded_name = if *deserialize_any_name {
                None
            } else {
                Some(expanded_name)
            };

            Self::element_visit_fn(
                ident,
                visitor_lifetime,
                expanded_name,
                data_struct,
                *children_order,
                *allow_unknown_children,
                *attribute_order,
                *allow_unknown_attributes,
            )?
        };

        let visitor_builder = VisitorBuilder::new(
            ident,
            generics,
            visitor_ident,
            visitor_lifetime,
            &formatter_expecting,
        )
        .visit_element_fn(visit_element_fn);

        let visitor_def = visitor_builder.definition();
        let visitor_trait_impl = visitor_builder.trait_impl();

        Ok(quote! {
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

    fn attribute_visit_fn(
        ident: &Ident,
        visitor_lifetime: &syn::Lifetime,
        expanded_name: Option<ExpandedName>,
        data_struct: &DataStruct,
    ) -> darling::Result<proc_macro2::TokenStream> {
        let attribute_access_ident = Ident::new("__attribute", ident.span());
        let xml_name_identification = expanded_name.map(|qname| {
        quote! {
            ::xmlity::de::AttributeAccessExt::ensure_name::<<A as ::xmlity::de::AttributeAccess<#visitor_lifetime>>::Error>(&#attribute_access_ident, &#qname)?;
        }
    });

        let deserialization_impl = match &data_struct.fields {
        syn::Fields::Unnamed(fields) if fields.unnamed.len() != 1 => simple_compile_error("Only tuple structs with 1 element are supported"),
        syn::Fields::Unnamed(_) => {
            quote! {
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

        Ok(VisitorBuilder::visit_attribute_fn_signature(
            &attribute_access_ident,
            visitor_lifetime,
            quote! {
                #xml_name_identification

                #deserialization_impl
            },
        ))
    }
}

impl DeserializeContent for StructAttributeVisitorBuilder<'_> {
    fn deserialize_content(
        &self,
        ast: &syn::DeriveInput,
        visitor_ident: &Ident,
        visitor_lifetime: &Lifetime,
        deserializer_ident: &Ident,
    ) -> Result<proc_macro2::TokenStream, DeriveError> {
        let data_struct = match ast.data {
            syn::Data::Struct(ref data_struct) => data_struct,
            _ => {
                return Err(darling::Error::custom("Only structs are supported").with_span(ast))?;
            }
        };

        let Self { opts } = self;

        let formatter_expecting = format!("struct {}", ast.ident);

        let crate::XmlityRootAttributeDeriveOpts {
            name,
            namespace,
            deserialize_any_name,
            ..
        } = opts;

        let visit_attribute_fn = {
            let ident_name = ast.ident.to_string();
            let expanded_name = if *deserialize_any_name {
                None
            } else {
                Some(ExpandedName::new(
                    name.0.as_ref().unwrap_or(&ident_name),
                    namespace.0.as_deref(),
                ))
            };

            Self::attribute_visit_fn(&ast.ident, visitor_lifetime, expanded_name, data_struct)?
        };

        let visitor_builder = VisitorBuilder::new(
            &ast.ident,
            &ast.generics,
            visitor_ident,
            visitor_lifetime,
            &formatter_expecting,
        )
        .visit_attribute_fn(visit_attribute_fn);

        let visitor_def = visitor_builder.definition();
        let visitor_trait_impl = visitor_builder.trait_impl();

        Ok(quote! {
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
) -> proc_macro2::TokenStream {
    let getter_declarations = attribute_fields
        .into_iter()
        .map(
            |DeserializeBuilderField {
                 builder_field_ident,
                 field_type,
                 ..
             }| {
                quote! {
                    let mut #builder_field_ident = ::core::option::Option::<#field_type>::None;
                }
            },
        )
        .chain(element_fields.into_iter().map(
            |DeserializeBuilderField {
                 builder_field_ident,
                 field_type,
                 ..
             }| {
                quote! {
                    let mut #builder_field_ident = ::core::option::Option::<#field_type>::None;
                }
            },
        )).chain(group_fields.into_iter().map(
            |DeserializeBuilderField {
                 builder_field_ident,
                 field_type,
                 ..
             }| {
                quote! {
                    let mut #builder_field_ident = <#field_type as ::xmlity::de::DeserializationGroup>::builder();
                }
            },
        ));

    quote! {
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
        .map(|DeserializeBuilderField { builder_field_ident, field_ident, options, .. }| {
            let expression = if matches!(options, XmlityFieldDeriveOpts::Element(XmlityFieldElementDeriveOpts {default: true}) | XmlityFieldDeriveOpts::Attribute(XmlityFieldAttributeDeriveOpts {default: true})) {
                quote! {
                    ::core::option::Option::unwrap_or_default(#builder_field_ident)
                }
            } else {
                quote! {
                    ::core::option::Option::ok_or(#builder_field_ident, ::xmlity::de::Error::missing_field(stringify!(#field_ident)))?
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
) -> proc_macro2::TokenStream {
    let field_visits = builder_attribute_field_visitor(
        access_ident,
        quote! {},
        fields.clone(),
        quote! {break;},
        match order {
            ElementOrder::Loose => quote! {break;},
            ElementOrder::None => quote! {continue;},
        },
        quote! {continue;},
        quote! {},
        match order {
            ElementOrder::Loose => true,
            ElementOrder::None => false,
        },
    );

    match order {
        ElementOrder::Loose => field_visits.zip(fields).map(|(field_visit, field)| {
            let skip_unknown = if allow_unknown_attributes {
                let skip_ident = Ident::new("__skip", span);
                quote! {
                    let #skip_ident = ::xmlity::de::AttributesAccess::next_attribute::<::xmlity::types::utils::IgnoredAny>(&mut #access_ident).unwrap_or(None);
                    if ::core::option::Option::is_none(&#skip_ident) {
                        break;
                    }
                    continue;
                }
            } else {
                let condition = attribute_done(field, quote! {});

                quote! {
                    if #condition {
                        break;
                    } else {
                        return ::core::result::Result::Err(::xmlity::de::Error::unknown_child());
                    }
                }
            };

            quote! {
                loop {
                    #field_visit
                    #skip_unknown
                }
            }
        }).collect(),
        ElementOrder::None => {
            let skip_unknown = if allow_unknown_attributes {
                let skip_ident = Ident::new("__skip", span);
                quote! {
                    let #skip_ident = ::xmlity::de::AttributesAccess::next_attribute::<::xmlity::types::utils::IgnoredAny>(&mut #access_ident).unwrap_or(None);
                    if ::core::option::Option::is_none(&#skip_ident) {
                        break
                    }
                }
            } else {
                let all_some_condition = all_attributes_done(fields, quote! {});

                quote! {
                    if #all_some_condition {
                        break;
                    } else {
                        return ::core::result::Result::Err(::xmlity::de::Error::unknown_child());
                    }
                }
            };

            quote! {
                loop {
                    #(#field_visits)*
                    #skip_unknown
                }
            }
        },
    }
}

struct SeqVisitLoop<'a, F: IntoIterator<Item = DeserializeBuilderField<FieldIdent, XmlityFieldElementGroupDeriveOpts>>
+ Clone> {
    seq_access_ident: &'a Ident,
    allow_unknown_children: bool,
    order: ElementOrder,
    fields: F,
}

impl<'a, F: IntoIterator<Item = DeserializeBuilderField<FieldIdent, XmlityFieldElementGroupDeriveOpts>>
+ Clone> SeqVisitLoop<'a, F> {
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

    fn access_loop(
        &self,
    ) -> proc_macro2::TokenStream {
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
            quote! {break;},
            match order {
                ElementOrder::Loose => quote! {break;},
                ElementOrder::None => quote! {continue;},
            },
            quote! {continue;},
            quote! {},
            match order {
                ElementOrder::Loose => true,
                ElementOrder::None => false,
            },
        );
    
         match order {
            ElementOrder::Loose => field_visits.zip(fields.clone()).map(|(field_visit, field)| {
                let skip_unknown = if *allow_unknown_children {
                    let skip_ident = Ident::new("__skip", access_ident.span());
                    quote! {
                        let #skip_ident = ::xmlity::de::SeqAccess::next_element::<::xmlity::types::utils::IgnoredAny>(&mut #access_ident).unwrap_or(None);
                        if ::core::option::Option::is_none(&#skip_ident) {
                            break;
                        }
                        continue;
                    }
                } else {
                    let condition = element_done(field, quote! {});
    
                    quote! {
                        if #condition {
                            break;
                        } else {
                            return ::core::result::Result::Err(::xmlity::de::Error::unknown_child());
                        }
                    }
                };
    
    
                quote! {
                    loop {
                        #field_visit
                        #skip_unknown
                    }
                }
            }).collect(),
            ElementOrder::None => {
                let skip_unknown = if *allow_unknown_children {
                    let skip_ident = Ident::new("__skip", access_ident.span());
                    quote! {
                        let #skip_ident = ::xmlity::de::SeqAccess::next_element::<::xmlity::types::utils::IgnoredAny>(&mut #access_ident).unwrap_or(None);
                        if ::core::option::Option::is_none(&#skip_ident) {
                            break
                        }
                    }
                } else {
                    let all_some_condition = all_elements_done(fields.clone(), quote! {});
    
                    quote! {
                        if #all_some_condition {
                            break;
                        } else {
                            return ::core::result::Result::Err(::xmlity::de::Error::unknown_child());
                        }
                    }
                };
    
                quote! {
                    loop {
                        #(#field_visits)*
                        #skip_unknown
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
) -> proc_macro2::TokenStream {
    let access_ident = Ident::new("__children", element_access_ident.span());

    let visit = SeqVisitLoop::new(&access_ident,  allow_unknown_children, order, fields);

    let field_storage = visit.field_storage();
    let access_loop = visit.access_loop();

    quote! {
        let mut #access_ident = ::xmlity::de::ElementAccess::children(#element_access_ident)?;

        #field_storage

        #access_loop
    }
}

struct EnumNoneVisitorBuilder {
}

impl EnumNoneVisitorBuilder {
    fn new() -> Self {
        Self {
        }
    }
}

impl DeserializeContent for EnumNoneVisitorBuilder {
    fn deserialize_content(
        &self,
        DeriveInput {
            ident,
            generics,
            data,
            ..
        }: &syn::DeriveInput,
        visitor_ident: &Ident,
        visitor_lifetime: &Lifetime,
        deserializer_ident: &Ident,
    ) -> Result<proc_macro2::TokenStream, DeriveError> {
        let data_enum = match &data {
            syn::Data::Enum(data_enum) => data_enum,
            _ => panic!("Wrong options. Only enums can be used for xelement."),
        };
        let variants = data_enum.variants.iter().collect::<Vec<_>>();

        let visit_seq_fn = (|| {
            let sequence_access_ident = Ident::new("__sequence", Span::mixed_site());
    
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
                            if let ::core::result::Result::Ok(::core::option::Option::Some(_v)) = ::xmlity::de::SeqAccess::next_element::<#field_type>(&mut #sequence_access_ident) {
                                return ::core::result::Result::Ok(#ident::#variant_ident(_v));
                            }
                        })
                    }
                    syn::Fields::Unit =>{
                        None
                    },
                }
            }).collect::<Option<Vec<_>>>()?;
            let ident_string = ident.to_string();
    
            Some(VisitorBuilder::visit_seq_fn_signature(
                &sequence_access_ident,
                visitor_lifetime,
                quote! {
                    #(#variants)*
    
                    ::core::result::Result::Err(::xmlity::de::Error::no_possible_variant(#ident_string))
                },
            ))
        })();
    
        let formatter_expecting = format!("enum {}", ident);
    
        let visitor_builder = VisitorBuilder::new(
            ident,
            generics,
            visitor_ident,
            visitor_lifetime,
            &formatter_expecting,
        )
        .visit_seq_fn(visit_seq_fn);
    
        let visitor_def = visitor_builder.definition();
        let visitor_trait_impl = visitor_builder.trait_impl();
    
        let deserialize_expr = quote! {
            ::xmlity::de::Deserializer::deserialize_seq(#deserializer_ident, #visitor_ident {
                lifetime: ::core::marker::PhantomData,
                marker: ::core::marker::PhantomData,
            })
        };
    
        Ok(quote! {
            #visitor_def
            #visitor_trait_impl
            #deserialize_expr
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
}

impl DeserializeContent for EnumValueVisitorBuilder<'_> {
    fn deserialize_content(
        &self,
        DeriveInput {  ident, generics, data, .. }: &syn::DeriveInput,
        visitor_ident: &Ident,
        visitor_lifetime: &Lifetime,
        deserializer_ident: &Ident,
    ) -> Result<proc_macro2::TokenStream, DeriveError>

{
    let syn::Data::Enum(DataEnum { variants, .. }) = data else {
        panic!("This is guaranteed by the caller");
    };

    let Self { opts } = self;

    let (visit_text_fn, visit_cdata_fn) = (|| {
        let value_ident_owned = Ident::new("__valueowned", Span::mixed_site());
        let value_ident = Ident::new("__value", Span::mixed_site());
        let variants = variants.clone().into_iter().map(|Variant {
            ident: variant_ident,
            fields: variant_fields,
            ..
        }| {
            let variant_ident_string = opts.rename_all.apply_to_variant(&variant_ident.to_string());
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
            None => return (None, None),
        };

        let ident_string = ident.to_string();

        let fn_body = quote! {
            #(#variants)*

            ::core::result::Result::Err(::xmlity::de::Error::no_possible_variant(#ident_string))
        };

        (
            Some(VisitorBuilder::visit_text_fn_signature(
                &value_ident_owned,
                quote! {
                    let #value_ident = ::xmlity::de::XmlText::as_str(&#value_ident_owned);
                    #fn_body
                },
            )),
            Some(VisitorBuilder::visit_cdata_fn_signature(
                &value_ident_owned,
                quote! {
                    let #value_ident = ::xmlity::de::XmlCData::as_str(&#value_ident_owned);
                    #fn_body
                },
            )),
        )
    })();

    let formatter_expecting = format!("enum {}", ident);

    let visitor_builder = VisitorBuilder::new(
        ident,
        generics,
        visitor_ident,
        visitor_lifetime,
        &formatter_expecting,
    )
    .visit_text_fn(visit_text_fn)
    .visit_cdata_fn(visit_cdata_fn);

    let visitor_def = visitor_builder.definition();
    let visitor_trait_impl = visitor_builder.trait_impl();

    let deserialize_expr = quote! {
        ::xmlity::de::Deserializer::deserialize_any(#deserializer_ident, #visitor_ident {
            lifetime: ::core::marker::PhantomData,
            marker: ::core::marker::PhantomData,
        })
    };

    Ok(quote! {
        #visitor_def
        #visitor_trait_impl
        #deserialize_expr
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
                    .deserialize_impl(ast)
            }
            (syn::Data::Struct(_), DeriveDeserializeOption::Attribute(opts)) => {
                StructAttributeVisitorBuilder::new(opts).deserialize_impl(ast)
            }
            (syn::Data::Struct(_), DeriveDeserializeOption::Value(_opts)) => {
                // struct_attribute_de_impl(
                //     &ast.ident,
                //     &ast.generics,
                //     &visitor_ident,
                //     &visitor_lifetime,
                //     &deserializer_ident,
                //     data_struct,
                //     attribute_opts,
                // )?
                Ok(simple_compile_error("Structs with value options are not supported yet"))
            }
            (syn::Data::Enum(_), DeriveDeserializeOption::None) => {
                EnumNoneVisitorBuilder::new().deserialize_impl(ast)
            }
            (
                syn::Data::Enum(_),
                DeriveDeserializeOption::Value(value_opts),
            ) => EnumValueVisitorBuilder::new(value_opts)
                .deserialize_impl(ast),
            (syn::Data::Union(_), _) => Ok(simple_compile_error("Unions are not supported yet")),
            _ => Ok(simple_compile_error("Wrong options")),
        }
    }
}