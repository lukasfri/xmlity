pub enum DeriveError {
    Darling(darling::Error),
    Custom {
        message: String,
        span: proc_macro2::Span,
    },
}

pub type DeriveResult<T> = Result<T, DeriveError>;

impl From<darling::Error> for DeriveError {
    fn from(e: darling::Error) -> Self {
        DeriveError::Darling(e)
    }
}

impl From<syn::Error> for DeriveError {
    fn from(e: syn::Error) -> Self {
        DeriveError::Darling(e.into())
    }
}

impl DeriveError {
    pub fn into_compile_error(self) -> proc_macro2::TokenStream {
        match self {
            DeriveError::Darling(e) => e.write_errors(),
            DeriveError::Custom { message, span } => {
                syn::Error::new(span, message).to_compile_error()
            }
        }
    }

    pub fn custom<T: Into<String>>(error: T) -> Self {
        Self::custom_with_span(error, proc_macro2::Span::call_site())
    }

    pub fn custom_with_span<T: Into<String>, S: Into<proc_macro2::Span>>(
        error: T,
        span: S,
    ) -> Self {
        Self::Custom {
            message: error.into(),
            span: span.into(),
        }
    }
}

pub trait DeriveMacro {
    fn input_to_derive(ast: &syn::DeriveInput) -> Result<proc_macro2::TokenStream, DeriveError>
    where
        Self: Sized;
}

pub trait DeriveMacroExt {
    fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream;
}

impl<T: DeriveMacro> DeriveMacroExt for T {
    fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
        let ast = syn::parse_macro_input!(input as syn::DeriveInput);
        T::input_to_derive(&ast)
            .unwrap_or_else(|e| e.into_compile_error())
            .into()
    }
}
