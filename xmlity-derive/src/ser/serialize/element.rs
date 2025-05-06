use std::borrow::Cow;

use proc_macro2::Span;
use quote::quote;
use syn::{parse_quote, Expr, Ident, Lifetime, Stmt};

use crate::common::value_deconstructor;
use crate::common::Prefix;
use crate::common::RecordInput;
use crate::common::StructTypeWithFields;
use crate::options::records;
use crate::options::records::fields::{ChildOpts, FieldOpts, ValueOpts};
use crate::options::WithExpandedNameExt;
use crate::options::{Extendable, FieldWithOpts};
use crate::ser::builders::SerializeBuilder;
use crate::ser::common::attribute_group_field_serializer;
use crate::ser::common::attribute_group_fields;
use crate::ser::common::element_group_field_serializer;
use crate::ser::common::element_group_fields;
use crate::{
    common::{ExpandedName, FieldIdent},
    DeriveError,
};

#[allow(clippy::type_complexity)]
pub struct SingleChildSerializeElementBuilder<'a> {
    pub ident: &'a syn::Ident,
    pub required_expanded_name: ExpandedName<'static>,
    pub preferred_prefix: Option<Prefix<'static>>,
    pub enforce_prefix: bool,
    pub item_type: &'a syn::Type,
}

impl SingleChildSerializeElementBuilder<'_> {
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

impl SerializeBuilder for SingleChildSerializeElementBuilder<'_> {
    fn serialize_fn_body(
        &self,
        serializer_access: &Ident,
        serializer_type: &syn::Type,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let ident = self.ident;

        let input = RecordInput {
            impl_for_ident: Cow::Borrowed(self.ident),
            constructor_path: Cow::Owned(parse_quote!(#ident)),
            result_type: Cow::Borrowed(self.item_type),
            generics: Cow::Owned(parse_quote!()),
            wrapper_function: std::convert::identity,
            record_path: Cow::Owned(parse_quote!(self)),
            fields: StructTypeWithFields::Named(vec![FieldWithOpts {
                field_ident: self.value_access_ident(),
                field_type: self.item_type.clone(),
                options: FieldOpts::Value(ChildOpts::Value(ValueOpts {
                    default: false,
                    extendable: Extendable::None,
                })),
            }]),
            sub_path_ident: None,
            fallable_deconstruction: false,
        };

        let builder = RecordSerializeElementBuilder {
            input: &input,
            required_expanded_name: self.required_expanded_name.clone(),
            preferred_prefix: self.preferred_prefix.clone(),
            enforce_prefix: self.enforce_prefix,
        };

        builder.serialize_fn_body(serializer_access, serializer_type)
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
pub struct RecordSerializeElementBuilder<'a, T: Fn(syn::Expr) -> syn::Expr> {
    // pub ident: &'a syn::Ident,
    // pub generics: &'a syn::Generics,
    pub required_expanded_name: ExpandedName<'static>,
    // pub struct_type: StructTypeWithFields<
    //     Vec<FieldWithOpts<syn::Ident, FieldOpts>>,
    //     Vec<FieldWithOpts<syn::Index, FieldOpts>>,
    // >,
    pub preferred_prefix: Option<Prefix<'static>>,
    pub enforce_prefix: bool,
    pub input: &'a RecordInput<'a, T>,
}

impl<'a, T: Fn(syn::Expr) -> syn::Expr> RecordSerializeElementBuilder<'a, T> {
    pub fn new(input: &'a RecordInput<'a, T>, opts: &'a records::roots::RootElementOpts) -> Self {
        let expanded_name = opts
            .expanded_name(&input.impl_for_ident.to_string())
            .into_owned();
        Self {
            input,
            preferred_prefix: opts.preferred_prefix.clone(),
            enforce_prefix: opts.enforce_prefix,
            required_expanded_name: expanded_name,
        }
    }
}

impl<T: Fn(syn::Expr) -> syn::Expr> SerializeBuilder for RecordSerializeElementBuilder<'_, T> {
    fn serialize_fn_body(
        &self,
        serializer_access: &Ident,
        _serializer_type: &syn::Type,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let Self {
            input,
            enforce_prefix,
            required_expanded_name,
            preferred_prefix,
            ..
        } = self;

        let record_path = self.input.record_path.as_ref();

        let value_deconstructor = value_deconstructor(
            self.input.constructor_path.as_ref(),
            &parse_quote!(&#record_path),
            &self.input.fields,
            self.input.fallable_deconstruction,
        );

        let element_seq_ident = Ident::new("__element", proc_macro2::Span::call_site());
        let children_seq_ident = Ident::new("__children", proc_macro2::Span::call_site());
        let xml_name_temp_ident = Ident::new("__xml_name", proc_macro2::Span::call_site());

        let fields = match &input.fields {
            StructTypeWithFields::Named(fields) => fields
                .iter()
                .cloned()
                .map(|a| a.map_ident(FieldIdent::Named))
                .collect::<Vec<_>>(),
            StructTypeWithFields::Unnamed(fields) => fields
                .iter()
                .cloned()
                .map(|a| a.map_ident(FieldIdent::Indexed))
                .collect::<Vec<_>>(),
            StructTypeWithFields::Unit => vec![],
        };
        let attribute_fields = attribute_group_fields(fields.clone())?;
        let element_fields = element_group_fields(fields)?;

        let attribute_fields = attribute_group_field_serializer(
            quote! {&mut #element_seq_ident},
            attribute_fields,
            |field_ident| {
                let ident_name = field_ident.to_named_ident();
                parse_quote!(#ident_name)
            },
        )?;

        let element_end = if element_fields.is_empty() {
            quote! {
                ::xmlity::ser::SerializeElement::end(#element_seq_ident)
            }
        } else {
            let element_fields = element_group_field_serializer(
                quote! {&mut #children_seq_ident},
                element_fields,
                |field_ident| {
                    let ident_name = field_ident.to_named_ident();
                    parse_quote!(#ident_name)
                },
            )?;

            quote! {
                let mut #children_seq_ident = ::xmlity::ser::SerializeElement::serialize_children(#element_seq_ident)?;
                #element_fields
                ::xmlity::ser::SerializeSeq::end(#children_seq_ident)
            }
        };

        let preferred_prefix_setting = preferred_prefix.as_ref().map::<Stmt, _>(|preferred_prefix| parse_quote! {
              ::xmlity::ser::SerializeElement::preferred_prefix(&mut #element_seq_ident, ::core::option::Option::Some(#preferred_prefix))?;
          });
        let enforce_prefix_setting = Some(*enforce_prefix).filter(|&enforce_prefix| enforce_prefix).map::<Stmt, _>(|enforce_prefix| parse_quote! {
              ::xmlity::ser::SerializeElement::include_prefix(&mut #element_seq_ident, #enforce_prefix)?;
          });

        Ok(parse_quote! {
            let #xml_name_temp_ident = #required_expanded_name;
            let mut #element_seq_ident = ::xmlity::Serializer::serialize_element(#serializer_access, &#xml_name_temp_ident)?;
            #(#value_deconstructor)*
            #preferred_prefix_setting
            #enforce_prefix_setting
            #attribute_fields
            #element_end
        })
    }

    fn ident(&self) -> Cow<'_, Ident> {
        Cow::Borrowed(self.input.impl_for_ident.as_ref())
    }

    fn generics(&self) -> Cow<'_, syn::Generics> {
        Cow::Borrowed(self.input.generics.as_ref())
    }
}
