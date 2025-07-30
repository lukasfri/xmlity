use std::borrow::Cow;

use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{parse_quote, Arm, Data, Expr, Lifetime, Stmt};
use syn::{DeriveInput, Ident};

use crate::common::{ExpandedName, Prefix, StructTypeWithFields};
use crate::options::records::roots::RootAttributeOpts;
use crate::options::{FieldWithOpts, WithExpandedNameExt};

use crate::DeriveError;
use crate::{DeriveMacro, DeriveResult};

use super::builders::{SerializeAttributeBuilder, SerializeAttributeBuilderExt};

pub struct SerializeAttributeStructUnnamedSingleFieldBuilder<'a> {
    ast: &'a syn::DeriveInput,
    opts: &'a RootAttributeOpts,
}

impl<'a> SerializeAttributeStructUnnamedSingleFieldBuilder<'a> {
    pub fn new(ast: &'a syn::DeriveInput, opts: &'a RootAttributeOpts) -> Self {
        Self { ast, opts }
    }

    pub fn to_builder(&self) -> DeriveResult<StructSerializeAttributeBuilder> {
        let DeriveInput {
            ident, generics, ..
        } = &self.ast;
        let RootAttributeOpts {
            enforce_prefix,
            preferred_prefix,
            ..
        } = self.opts;

        let expanded_name = self
            .opts
            .expanded_name(ident.to_string().as_str())
            .into_owned();
        let Data::Struct(data_struct) = &self.ast.data else {
            unreachable!()
        };

        let struct_type = match &data_struct.fields {
            syn::Fields::Named(fields_named) if fields_named.named.len() != 1 => {
                return Err(DeriveError::custom(format!(
                    "Expected a single field for attribute deserialization, found {}",
                    fields_named.named.len()
                )))
            }
            syn::Fields::Named(fields_named) => {
                let field = &fields_named.named[0];
                let field_ident = field.ident.as_ref().unwrap().clone();
                let field_type = field.ty.clone();
                StructTypeWithFields::Named(FieldWithOpts {
                    field_ident,
                    field_type,
                    options: (),
                })
            }
            syn::Fields::Unnamed(fields_unnamed) if fields_unnamed.unnamed.len() != 1 => {
                return Err(DeriveError::custom(format!(
                    "Expected a single field for attribute deserialization, found {}",
                    fields_unnamed.unnamed.len()
                )))
            }
            syn::Fields::Unnamed(fields_unnamed) => {
                let field = &fields_unnamed.unnamed[0];
                let field_type = field.ty.clone();
                StructTypeWithFields::Unnamed(FieldWithOpts {
                    field_ident: syn::Index::from(0),
                    field_type,
                    options: (),
                })
            }
            syn::Fields::Unit => StructTypeWithFields::Unit,
        };

        Ok(StructSerializeAttributeBuilder {
            ident,
            generics,
            expanded_name,
            struct_type,
            preferred_prefix: preferred_prefix.clone(),
            enforce_prefix: *enforce_prefix,
        })
    }
}

pub struct SimpleSerializeAttributeBuilder<'a> {
    pub ident: &'a syn::Ident,
    pub generics: &'a syn::Generics,
    pub item_type: &'a syn::Type,
    pub expanded_name: ExpandedName<'static>,
    pub preferred_prefix: Option<Prefix<'static>>,
    pub enforce_prefix: bool,
}

impl SimpleSerializeAttributeBuilder<'_> {
    fn value_access_ident(&self) -> Ident {
        Ident::new("__value", Span::call_site())
    }

    fn value_lifetime(&self) -> Lifetime {
        Lifetime::new("'__value", Span::call_site())
    }

    pub fn struct_definition(&self) -> syn::ItemStruct {
        let Self {
            ident, item_type, ..
        } = self;

        let value_access_ident = self.value_access_ident();
        let generics = self.generics();
        let lifetime = self.value_lifetime();

        parse_quote! {
            struct #ident #generics {
                #value_access_ident: &#lifetime #item_type,
            }
        }
    }

    pub fn value_expression(&self, value_expr: &Expr) -> syn::Expr {
        let Self { ident, .. } = self;
        let value_access_ident = self.value_access_ident();
        parse_quote! {
            #ident {
                #value_access_ident: #value_expr,
            }
        }
    }
}

impl SerializeAttributeBuilder for SimpleSerializeAttributeBuilder<'_> {
    fn serialize_attribute_fn_body(
        &self,
        serializer_access: &Ident,
        serializer_type: &syn::Type,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let builder = StructSerializeAttributeBuilder {
            ident: self.ident,
            generics: self.generics,

            struct_type: StructTypeWithFields::Named(FieldWithOpts {
                field_ident: self.value_access_ident(),
                field_type: self.item_type.clone(),
                options: (),
            }),
            expanded_name: self.expanded_name.clone(),
            preferred_prefix: self.preferred_prefix.clone(),
            enforce_prefix: self.enforce_prefix,
        };

        builder.serialize_attribute_fn_body(serializer_access, serializer_type)
    }

    fn ident(&self) -> Cow<'_, Ident> {
        Cow::Borrowed(self.ident)
    }

    fn generics(&self) -> Cow<'_, syn::Generics> {
        let lifetime = self.value_lifetime();
        let generics = parse_quote!(<#lifetime>);
        Cow::Owned(generics)
    }
}

#[allow(clippy::type_complexity)]
pub struct StructSerializeAttributeBuilder<'a> {
    pub ident: &'a syn::Ident,
    pub generics: &'a syn::Generics,
    pub expanded_name: ExpandedName<'static>,
    pub preferred_prefix: Option<Prefix<'static>>,
    pub enforce_prefix: bool,
    pub struct_type:
        StructTypeWithFields<FieldWithOpts<syn::Ident, ()>, FieldWithOpts<syn::Index, ()>>,
}

impl SerializeAttributeBuilder for StructSerializeAttributeBuilder<'_> {
    fn serialize_attribute_fn_body(
        &self,
        serializer_access: &Ident,
        _serializer_type: &syn::Type,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let Self {
            preferred_prefix,
            enforce_prefix,
            expanded_name,
            struct_type,
            ..
        } = self;
        let value_exepr = match struct_type {
            StructTypeWithFields::Named(FieldWithOpts { field_ident, .. }) => {
                quote! {
                    &self.#field_ident
                }
            }
            StructTypeWithFields::Unnamed(FieldWithOpts { field_ident, .. }) => {
                quote! {
                    &self.#field_ident
                }
            }
            StructTypeWithFields::Unit => {
                quote! {
                    &""
                }
            }
        };

        let access_ident = Ident::new("__sa", proc_macro2::Span::call_site());
        let xml_name_temp_ident = Ident::new("__xml_name", proc_macro2::Span::call_site());

        let preferred_prefix_setting = preferred_prefix.as_ref().map::<Stmt, _>(|preferred_prefix| parse_quote! {
            ::xmlity::ser::SerializeAttributeAccess::preferred_prefix(&mut #access_ident, ::core::option::Option::Some(#preferred_prefix))?;
        });
        let enforce_prefix_setting = enforce_prefix.then(|| {
            parse_quote!(::xmlity::ser::IncludePrefix::WhenNecessaryForPreferredPrefix)
        }).map::<Stmt, _>(|enforce_prefix: syn::Expr| parse_quote! {
            ::xmlity::ser::SerializeAttributeAccess::include_prefix(&mut #access_ident, #enforce_prefix)?;
        });

        Ok(parse_quote! {
            let #xml_name_temp_ident = #expanded_name;
            let mut #access_ident = ::xmlity::AttributeSerializer::serialize_attribute(
                &mut #serializer_access,
                &#xml_name_temp_ident,
            )?;
            #preferred_prefix_setting
            #enforce_prefix_setting
            ::xmlity::ser::SerializeAttributeAccess::end(#access_ident, #value_exepr)
        })
    }

    fn ident(&self) -> Cow<'_, Ident> {
        Cow::Borrowed(self.ident)
    }

    fn generics(&self) -> Cow<'_, syn::Generics> {
        Cow::Borrowed(self.generics)
    }
}

pub struct EnumSingleFieldAttributeSerializeBuilder<'a> {
    ast: &'a syn::DeriveInput,
}

impl<'a> EnumSingleFieldAttributeSerializeBuilder<'a> {
    pub fn new(ast: &'a syn::DeriveInput) -> Self {
        Self { ast }
    }
}

impl SerializeAttributeBuilder for EnumSingleFieldAttributeSerializeBuilder<'_> {
    fn serialize_attribute_fn_body(
        &self,
        serializer_access: &Ident,
        _serializer_type: &syn::Type,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let DeriveInput { ident, data, .. } = self.ast;
        let syn::Data::Enum(syn::DataEnum { variants, .. }) = data else {
            unreachable!()
        };

        let variants = variants
            .iter()
            .map::<Result<Arm, DeriveError>, _>(|variant| {
                let variant_name = &variant.ident;
                match &variant.fields {
                    syn::Fields::Named(_fields) => {
                        Err(DeriveError::custom("Named fields are not supported yet"))
                    }
                    syn::Fields::Unnamed(fields) => {
                        if fields.unnamed.len() == 1 {
                            Ok(parse_quote! {
                                #ident::#variant_name(val) => {
                                    ::xmlity::Serialize::serialize(&val, &mut #serializer_access)
                                },
                            })
                        } else {
                            Err(DeriveError::custom(
                                "Enum variants with more than one field are not supported",
                            ))
                        }
                    }
                    syn::Fields::Unit => {
                        Err(DeriveError::custom("Unit variants are not supported yet"))
                    }
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(parse_quote!(
            match self {
                #(#variants)*
            }
        ))
    }

    fn ident(&self) -> Cow<'_, Ident> {
        Cow::Borrowed(&self.ast.ident)
    }

    fn generics(&self) -> Cow<'_, syn::Generics> {
        Cow::Borrowed(&self.ast.generics)
    }
}

enum SerializeAttributeOption {
    Attribute(RootAttributeOpts),
}

impl SerializeAttributeOption {
    pub fn parse(ast: &DeriveInput) -> Result<Self, DeriveError> {
        let attribute_opts = RootAttributeOpts::parse(&ast.attrs)?.ok_or_else(|| {
            DeriveError::custom("SerializeAttribute requires the `xattribute` option.")
        })?;

        Ok(SerializeAttributeOption::Attribute(attribute_opts))
    }
}

pub struct DeriveSerializeAttribute;

impl DeriveMacro for DeriveSerializeAttribute {
    fn input_to_derive(ast: &DeriveInput) -> Result<proc_macro2::TokenStream, DeriveError> {
        let SerializeAttributeOption::Attribute(opts) = SerializeAttributeOption::parse(ast)?;

        match &ast.data {
            syn::Data::Struct(syn::DataStruct { fields, .. }) => match fields {
                syn::Fields::Unnamed(fields) if fields.unnamed.len() != 1 => Err(
                    DeriveError::custom("Structs with more than one field are not supported."),
                ),
                syn::Fields::Unnamed(_) => {
                    SerializeAttributeStructUnnamedSingleFieldBuilder::new(ast, &opts)
                        .to_builder()?
                        .serialize_attribute_trait_impl()
                        .map(|x| x.to_token_stream())
                }
                syn::Fields::Named(_) => {
                    Err(DeriveError::custom("Named fields are not supported yet."))
                }
                syn::Fields::Unit => {
                    Err(DeriveError::custom("Unit structs are not supported yet."))
                }
            },
            syn::Data::Enum(_) => EnumSingleFieldAttributeSerializeBuilder::new(ast)
                .serialize_attribute_trait_impl()
                .map(|x| x.to_token_stream()),
            syn::Data::Union(_) => Err(DeriveError::custom(
                "Unions are not supported for serialization to attributes.",
            )),
        }
    }
}
