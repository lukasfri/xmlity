use crate::define_test;

use xmlity::{
    DeserializationGroup, Deserialize, DeserializeOwned, SerializationGroup, Serialize,
    SerializeAttribute,
};

#[derive(Debug, PartialEq, SerializeAttribute, Deserialize)]
#[xattribute(name = "a")]
pub struct A(String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "b")]
pub struct B<T: SerializeAttribute + DeserializeOwned> {
    #[xattribute(deferred = true)]
    pub a: T,
}

define_test!(
    generic_element,
    [(
        B {
            a: A("A".to_string()),
        },
        r#"<b a="A"/>"#
    )]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum C<T: SerializeAttribute + DeserializeOwned> {
    B(B<T>),
}

define_test!(
    generic_enum,
    [(
        C::B(B {
            a: A("A".to_string()),
        }),
        r#"<b a="A"/>"#
    )]
);

#[derive(Debug, PartialEq, SerializationGroup, DeserializationGroup)]
pub struct D<T: SerializeAttribute + DeserializeOwned> {
    #[xattribute(deferred = true)]
    pub c: T,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "d")]
pub struct E {
    #[xgroup]
    pub c: D<A>,
}

define_test!(
    generic_group,
    [(
        E {
            c: D {
                c: A("A".to_string()),
            },
        },
        r#"<d a="A"/>"#
    )]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum F<T: Serialize + DeserializeOwned, U: Serialize + DeserializeOwned> {
    T(T),
    U(U),
}

define_test!(
    two_armed_generic_enum,
    [
        (F::<String, String>::T("A".to_string()), r#"A"#),
        (F::<u32, f32>::U(0.5), r#"0.5"#)
    ]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum G<T: Serialize + DeserializeOwned, U: Serialize + DeserializeOwned> {
    #[xelement(name = "t")]
    T(T),
    #[xelement(name = "u")]
    U(U),
}

define_test!(
    two_armed_element_generic_enum,
    [
        (G::<String, String>::T("A".to_string()), r#"<t>A</t>"#),
        (G::<u32, f32>::U(0.5), r#"<u>0.5</u>"#)
    ]
);
