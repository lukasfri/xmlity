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
    #[xattribute]
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
pub enum BEnum<T: SerializeAttribute + DeserializeOwned> {
    B(B<T>),
}

define_test!(
    generic_enum,
    [(
        BEnum::B(B {
            a: A("A".to_string()),
        }),
        r#"<b a="A"/>"#
    )]
);

#[derive(Debug, PartialEq, SerializationGroup, DeserializationGroup)]
pub struct C<T: SerializeAttribute + DeserializeOwned> {
    #[xattribute]
    pub c: T,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "d")]
pub struct D {
    #[xgroup]
    pub c: C<A>,
}

define_test!(
    generic_group,
    [(
        D {
            c: C {
                c: A("A".to_string()),
            },
        },
        r#"<d a="A"/>"#
    )]
);
