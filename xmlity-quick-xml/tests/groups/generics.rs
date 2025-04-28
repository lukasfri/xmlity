use crate::define_test;

use xmlity::{
    DeserializationGroup, Deserialize, DeserializeOwned, SerializationGroup, Serialize,
    SerializeAttribute,
};

#[derive(Debug, PartialEq, SerializeAttribute, Deserialize)]
#[xattribute(name = "to")]
pub struct To(String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "note")]
pub struct Note<T: SerializeAttribute + DeserializeOwned> {
    #[xgroup]
    pub to: NoteGroup<T>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum NoteEnum<T: SerializeAttribute + DeserializeOwned> {
    Note(Note<T>),
}

#[derive(Debug, PartialEq, SerializationGroup, DeserializationGroup)]
pub struct NoteGroup<T: SerializeAttribute + DeserializeOwned> {
    #[xattribute]
    pub to: T,
}

fn simple_2d_struct_result() -> Note<To> {
    Note {
        to: NoteGroup {
            to: To("Tove".to_string()),
        },
    }
}

define_test!(
    generic_group,
    [(simple_2d_struct_result(), r#"<note to="Tove"/>"#)]
);
