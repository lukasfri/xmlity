use std::convert::Infallible;

impl crate::de::Error for Infallible {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        panic!("Infallible error: {msg}");
    }

    fn wrong_name(name: &crate::ExpandedName<'_>, expected: &crate::ExpandedName<'_>) -> Self {
        panic!("Infallible error: wrong name \"{name}\", expected \"{expected}\"");
    }

    fn unexpected_visit<T>(unexpected: crate::de::Unexpected, _expected: &T) -> Self {
        panic!("Infallible error: unexpected visit of {unexpected}");
    }

    fn missing_field(field: &str) -> Self {
        panic!("Infallible error: missing field {field}");
    }

    fn no_possible_variant(ident: &str) -> Self {
        panic!("Infallible error: no possible variant {ident}");
    }

    fn missing_data() -> Self {
        panic!("Infallible error: missing data");
    }

    fn unknown_child() -> Self {
        panic!("Infallible error: unknown child");
    }

    fn invalid_string() -> Self {
        panic!("Infallible error: invalid string");
    }
}

impl crate::ser::Error for Infallible {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        panic!("Infallible error: {msg}");
    }

    fn unexpected_serialize(unexpected: crate::ser::Unexpected) -> Self {
        panic!("Infallible error: unexpected serialize of {unexpected}");
    }
}
