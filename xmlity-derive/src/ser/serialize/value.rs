use syn::{parse_quote, Arm, Data, DataEnum, DeriveInput, Ident, Stmt};

use crate::options::XmlityRootValueDeriveOpts;
use crate::DeriveError;

use super::SerializeBuilder;

pub struct DeriveValueEnum<'a> {
    opts: &'a XmlityRootValueDeriveOpts,
}

impl<'a> DeriveValueEnum<'a> {
    pub fn new(opts: &'a XmlityRootValueDeriveOpts) -> Self {
        Self { opts }
    }
}

impl SerializeBuilder for DeriveValueEnum<'_> {
    fn serialize_fn_body(
        &self,
        ast: &syn::DeriveInput,
        serializer_access: &Ident,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let DeriveInput { ident, data, .. } = ast;

        let Data::Enum(DataEnum { variants, .. }) = &data else {
            unreachable!()
        };

        let variants = variants
          .iter()
          .map::<Result<Arm, DeriveError>, _>(|variant| {
              let variant_ident = &variant.ident;
              let variant_ident_string = self
                  .opts
                  .rename_all
                  .apply_to_variant(&variant_ident.to_string());

              match &variant.fields {
                  syn::Fields::Named(_) | syn::Fields::Unnamed(_) => Err(DeriveError::custom(
                      "Unsupported named/unnamed field variant in value enum.",
                  )),
                  syn::Fields::Unit => Ok(parse_quote! {
                      #ident::#variant_ident => {
                          ::xmlity::Serialize::serialize(&#variant_ident_string, #serializer_access)
                      },
                  }),
              }
          })
          .collect::<Result<Vec<_>, _>>()?;

        Ok(parse_quote! {
            match self {
                #(#variants)*
            }
        })
    }
}
