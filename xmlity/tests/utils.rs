use rstest::rstest;
use std::{borrow::Cow, fmt::Debug};
use xmlity::{
    types::{
        utils::{ValueOrWhitespace, Whitespace},
        value::from_value,
    },
    xml, DeserializeOwned, XmlValue,
};

#[rstest]
#[case(xml!(""), Whitespace(Cow::Borrowed("")))]
fn whitespace<T: Into<XmlValue>>(#[case] input: T, #[case] expected: Whitespace) {
    let input = input.into();
    let actual = from_value::<Whitespace>(&input).unwrap();

    pretty_assertions::assert_eq!(expected, actual);
}

#[rstest]
#[case(xml!(""), ValueOrWhitespace::<()>::Whitespace(Cow::Borrowed("")))]
#[case(xml!("Text"), ValueOrWhitespace::<String>::Value("Text".to_owned()))]
fn value_or_whitespace<T: Into<XmlValue>, U: DeserializeOwned + PartialEq + Debug>(
    #[case] input: T,
    #[case] expected: ValueOrWhitespace<U>,
) {
    let input = input.into();
    let actual = from_value::<ValueOrWhitespace<U>>(&input).unwrap();

    pretty_assertions::assert_eq!(expected, actual);
}
