pub mod w3_attributes {
    pub mod actuate_items {
        #[derive(Debug, Clone, Copy, ::xmlity::Serialize, ::xmlity::Deserialize, PartialEq)]
        #[xvalue(with = actuate_with)]
        pub enum Actuate {
            OnLoad,
            OnRequest,
            Other,
            None,
        }
        pub mod actuate_with {
            pub fn deserialize<'de, D>(
                deserializer: D,
            ) -> ::core::result::Result<super::Actuate, D::Error>
            where
                D: ::xmlity::Deserializer<'de>,
            {
                let text: String = ::xmlity::Deserialize::deserialize(deserializer)?;
                let value: String = text.parse().map_err(::xmlity::de::Error::custom)?;
                super::Actuate::try_from(value).map_err(::xmlity::de::Error::custom)
            }
            pub fn serialize<S>(
                value: &super::Actuate,
                serializer: S,
            ) -> ::core::result::Result<S::Ok, S::Error>
            where
                S: ::xmlity::Serializer,
            {
                let value: String = Clone::clone(value).into();
                ::xmlity::Serialize::serialize(
                    String::as_str(&ToString::to_string(&value)),
                    serializer,
                )
            }
        }
        #[derive(Debug)]
        pub enum ActuateParseError {
            NonExistent { value: String },
        }
        impl ::core::fmt::Display for ActuateParseError {
            fn fmt(
                &self,
                f: &mut ::core::fmt::Formatter<'_>,
            ) -> ::core::result::Result<(), ::core::fmt::Error> {
                match self {
                    ActuateParseError::NonExistent { value } => {
                        write!(f, "Value '{:?}' does not exist in the enumeration", value)
                    }
                }
            }
        }
        impl ::core::convert::TryFrom<String> for Actuate {
            type Error = ActuateParseError;
            fn try_from(value: String) -> ::core::result::Result<Self, Self::Error> {
                match String::as_str(&value) {
                    "onLoad" => Ok(Actuate::OnLoad),
                    "onRequest" => Ok(Actuate::OnRequest),
                    "other" => Ok(Actuate::Other),
                    "none" => Ok(Actuate::None),
                    _ => Err(ActuateParseError::NonExistent { value }),
                }
            }
        }
        impl ::core::convert::From<Actuate> for String {
            fn from(value: Actuate) -> Self {
                match value {
                    Actuate::OnLoad => String::from("onLoad"),
                    Actuate::OnRequest => String::from("onRequest"),
                    Actuate::Other => String::from("other"),
                    Actuate::None => String::from("none"),
                }
            }
        }
    }
    #[derive(Debug, ::xmlity::SerializeAttribute, ::xmlity::Deserialize, PartialEq, Clone)]
    #[xattribute(name = "actuate", namespace = "http://www.w3.org/1999/xlink")]
    pub struct Actuate(pub actuate_items::Actuate);
    impl ::core::convert::From<actuate_items::Actuate> for Actuate {
        fn from(value: actuate_items::Actuate) -> Self {
            Actuate(value)
        }
    }
    pub mod arcrole_items {
        impl ::core::convert::From<String> for Arcrole {
            fn from(value: String) -> Self {
                Arcrole(value)
            }
        }
        #[derive(Debug, ::xmlity::Serialize, ::xmlity::Deserialize, PartialEq, Clone)]
        #[xvalue(with = arcrole_with)]
        pub struct Arcrole(pub String);
        pub mod arcrole_with {
            pub fn deserialize<'de, D>(
                deserializer: D,
            ) -> ::core::result::Result<super::Arcrole, D::Error>
            where
                D: ::xmlity::Deserializer<'de>,
            {
                let text: String = ::xmlity::Deserialize::deserialize(deserializer)?;
                let value: String = text.parse().map_err(::xmlity::de::Error::custom)?;
                super::Arcrole::try_from(value).map_err(::xmlity::de::Error::custom)
            }
            pub fn serialize<S>(
                value: &super::Arcrole,
                serializer: S,
            ) -> ::core::result::Result<S::Ok, S::Error>
            where
                S: ::xmlity::Serializer,
            {
                let value: String = Clone::clone(value).into();
                ::xmlity::Serialize::serialize(
                    String::as_str(&ToString::to_string(&value)),
                    serializer,
                )
            }
        }
        #[derive(Debug, PartialEq, Clone)]
        pub enum ArcroleParseError {}
        impl ::core::convert::From<Arcrole> for String {
            fn from(value: Arcrole) -> Self {
                value.0
            }
        }
    }
    #[derive(Debug, ::xmlity::SerializeAttribute, ::xmlity::Deserialize, PartialEq, Clone)]
    #[xattribute(name = "arcrole", namespace = "http://www.w3.org/1999/xlink")]
    pub struct Arcrole(pub arcrole_items::Arcrole);
    impl ::core::convert::From<arcrole_items::Arcrole> for Arcrole {
        fn from(value: arcrole_items::Arcrole) -> Self {
            Arcrole(value)
        }
    }
    #[derive(Debug, ::xmlity::SerializeAttribute, ::xmlity::Deserialize, PartialEq, Clone)]
    #[xattribute(name = "from", namespace = "http://www.w3.org/1999/xlink")]
    pub struct From(pub String);
    impl ::core::convert::From<String> for From {
        fn from(value: String) -> Self {
            From(value)
        }
    }
    #[derive(Debug, ::xmlity::SerializeAttribute, ::xmlity::Deserialize, PartialEq, Clone)]
    #[xattribute(name = "href", namespace = "http://www.w3.org/1999/xlink")]
    pub struct Href(pub String);
    impl ::core::convert::From<String> for Href {
        fn from(value: String) -> Self {
            Href(value)
        }
    }
    #[derive(Debug, ::xmlity::SerializeAttribute, ::xmlity::Deserialize, PartialEq, Clone)]
    #[xattribute(name = "label", namespace = "http://www.w3.org/1999/xlink")]
    pub struct Label(pub String);
    impl ::core::convert::From<String> for Label {
        fn from(value: String) -> Self {
            Label(value)
        }
    }
    pub mod role_items {
        impl ::core::convert::From<String> for Role {
            fn from(value: String) -> Self {
                Role(value)
            }
        }
        #[derive(Debug, ::xmlity::Serialize, ::xmlity::Deserialize, PartialEq, Clone)]
        #[xvalue(with = role_with)]
        pub struct Role(pub String);
        pub mod role_with {
            pub fn deserialize<'de, D>(
                deserializer: D,
            ) -> ::core::result::Result<super::Role, D::Error>
            where
                D: ::xmlity::Deserializer<'de>,
            {
                let text: String = ::xmlity::Deserialize::deserialize(deserializer)?;
                let value: String = text.parse().map_err(::xmlity::de::Error::custom)?;
                super::Role::try_from(value).map_err(::xmlity::de::Error::custom)
            }
            pub fn serialize<S>(
                value: &super::Role,
                serializer: S,
            ) -> ::core::result::Result<S::Ok, S::Error>
            where
                S: ::xmlity::Serializer,
            {
                let value: String = Clone::clone(value).into();
                ::xmlity::Serialize::serialize(
                    String::as_str(&ToString::to_string(&value)),
                    serializer,
                )
            }
        }
        #[derive(Debug, PartialEq, Clone)]
        pub enum RoleParseError {}
        impl ::core::convert::From<Role> for String {
            fn from(value: Role) -> Self {
                value.0
            }
        }
    }
    #[derive(Debug, ::xmlity::SerializeAttribute, ::xmlity::Deserialize, PartialEq, Clone)]
    #[xattribute(name = "role", namespace = "http://www.w3.org/1999/xlink")]
    pub struct Role(pub role_items::Role);
    impl ::core::convert::From<role_items::Role> for Role {
        fn from(value: role_items::Role) -> Self {
            Role(value)
        }
    }
    pub mod show_items {
        #[derive(Debug, Clone, Copy, ::xmlity::Serialize, ::xmlity::Deserialize, PartialEq)]
        #[xvalue(with = show_with)]
        pub enum Show {
            New,
            Replace,
            Embed,
            Other,
            None,
        }
        pub mod show_with {
            pub fn deserialize<'de, D>(
                deserializer: D,
            ) -> ::core::result::Result<super::Show, D::Error>
            where
                D: ::xmlity::Deserializer<'de>,
            {
                let text: String = ::xmlity::Deserialize::deserialize(deserializer)?;
                let value: String = text.parse().map_err(::xmlity::de::Error::custom)?;
                super::Show::try_from(value).map_err(::xmlity::de::Error::custom)
            }
            pub fn serialize<S>(
                value: &super::Show,
                serializer: S,
            ) -> ::core::result::Result<S::Ok, S::Error>
            where
                S: ::xmlity::Serializer,
            {
                let value: String = Clone::clone(value).into();
                ::xmlity::Serialize::serialize(
                    String::as_str(&ToString::to_string(&value)),
                    serializer,
                )
            }
        }
        #[derive(Debug)]
        pub enum ShowParseError {
            NonExistent { value: String },
        }
        impl ::core::fmt::Display for ShowParseError {
            fn fmt(
                &self,
                f: &mut ::core::fmt::Formatter<'_>,
            ) -> ::core::result::Result<(), ::core::fmt::Error> {
                match self {
                    ShowParseError::NonExistent { value } => {
                        write!(f, "Value '{:?}' does not exist in the enumeration", value)
                    }
                }
            }
        }
        impl ::core::convert::TryFrom<String> for Show {
            type Error = ShowParseError;
            fn try_from(value: String) -> ::core::result::Result<Self, Self::Error> {
                match String::as_str(&value) {
                    "new" => Ok(Show::New),
                    "replace" => Ok(Show::Replace),
                    "embed" => Ok(Show::Embed),
                    "other" => Ok(Show::Other),
                    "none" => Ok(Show::None),
                    _ => Err(ShowParseError::NonExistent { value }),
                }
            }
        }
        impl ::core::convert::From<Show> for String {
            fn from(value: Show) -> Self {
                match value {
                    Show::New => String::from("new"),
                    Show::Replace => String::from("replace"),
                    Show::Embed => String::from("embed"),
                    Show::Other => String::from("other"),
                    Show::None => String::from("none"),
                }
            }
        }
    }
    #[derive(Debug, ::xmlity::SerializeAttribute, ::xmlity::Deserialize, PartialEq, Clone)]
    #[xattribute(name = "show", namespace = "http://www.w3.org/1999/xlink")]
    pub struct Show(pub show_items::Show);
    impl ::core::convert::From<show_items::Show> for Show {
        fn from(value: show_items::Show) -> Self {
            Show(value)
        }
    }
    #[derive(Debug, ::xmlity::SerializeAttribute, ::xmlity::Deserialize, PartialEq, Clone)]
    #[xattribute(name = "title", namespace = "http://www.w3.org/1999/xlink")]
    pub struct Title(pub String);
    impl ::core::convert::From<String> for Title {
        fn from(value: String) -> Self {
            Title(value)
        }
    }
    #[derive(Debug, ::xmlity::SerializeAttribute, ::xmlity::Deserialize, PartialEq, Clone)]
    #[xattribute(name = "to", namespace = "http://www.w3.org/1999/xlink")]
    pub struct To(pub String);
    impl ::core::convert::From<String> for To {
        fn from(value: String) -> Self {
            To(value)
        }
    }
    pub mod type_items {
        #[derive(Debug, Clone, Copy, ::xmlity::Serialize, ::xmlity::Deserialize, PartialEq)]
        #[xvalue(with = type_with)]
        pub enum Type {
            Simple,
            Extended,
            Locator,
            Arc,
            Resource,
            Title,
        }
        pub mod type_with {
            pub fn deserialize<'de, D>(
                deserializer: D,
            ) -> ::core::result::Result<super::Type, D::Error>
            where
                D: ::xmlity::Deserializer<'de>,
            {
                let text: String = ::xmlity::Deserialize::deserialize(deserializer)?;
                let value: String = text.parse().map_err(::xmlity::de::Error::custom)?;
                super::Type::try_from(value).map_err(::xmlity::de::Error::custom)
            }
            pub fn serialize<S>(
                value: &super::Type,
                serializer: S,
            ) -> ::core::result::Result<S::Ok, S::Error>
            where
                S: ::xmlity::Serializer,
            {
                let value: String = Clone::clone(value).into();
                ::xmlity::Serialize::serialize(
                    String::as_str(&ToString::to_string(&value)),
                    serializer,
                )
            }
        }
        #[derive(Debug)]
        pub enum TypeParseError {
            NonExistent { value: String },
        }
        impl ::core::fmt::Display for TypeParseError {
            fn fmt(
                &self,
                f: &mut ::core::fmt::Formatter<'_>,
            ) -> ::core::result::Result<(), ::core::fmt::Error> {
                match self {
                    TypeParseError::NonExistent { value } => {
                        write!(f, "Value '{:?}' does not exist in the enumeration", value)
                    }
                }
            }
        }
        impl ::core::convert::TryFrom<String> for Type {
            type Error = TypeParseError;
            fn try_from(value: String) -> ::core::result::Result<Self, Self::Error> {
                match String::as_str(&value) {
                    "simple" => Ok(Type::Simple),
                    "extended" => Ok(Type::Extended),
                    "locator" => Ok(Type::Locator),
                    "arc" => Ok(Type::Arc),
                    "resource" => Ok(Type::Resource),
                    "title" => Ok(Type::Title),
                    _ => Err(TypeParseError::NonExistent { value }),
                }
            }
        }
        impl ::core::convert::From<Type> for String {
            fn from(value: Type) -> Self {
                match value {
                    Type::Simple => String::from("simple"),
                    Type::Extended => String::from("extended"),
                    Type::Locator => String::from("locator"),
                    Type::Arc => String::from("arc"),
                    Type::Resource => String::from("resource"),
                    Type::Title => String::from("title"),
                }
            }
        }
    }
    #[derive(Debug, ::xmlity::SerializeAttribute, ::xmlity::Deserialize, PartialEq, Clone)]
    #[xattribute(name = "type", namespace = "http://www.w3.org/1999/xlink")]
    pub struct Type(pub type_items::Type);
    impl ::core::convert::From<type_items::Type> for Type {
        fn from(value: type_items::Type) -> Self {
            Type(value)
        }
    }
}

pub mod linkbase_ref_items {
    #[derive(
        Debug, ::xmlity::SerializationGroup, ::xmlity::DeserializationGroup, PartialEq, Clone,
    )]
    pub struct LinkbaseRef {
        #[xattribute(deferred = true)]
        pub type_: Box<super::w3_attributes::Type>,
        #[xattribute(deferred = true)]
        pub href: Box<super::w3_attributes::Href>,
        #[xattribute(deferred = true)]
        pub arcrole: Box<super::w3_attributes::Arcrole>,
        #[xattribute(deferred = true, optional)]
        pub role: Option<Box<super::w3_attributes::Role>>,
        #[xattribute(deferred = true, optional)]
        pub title: Option<Box<super::w3_attributes::Title>>,
        #[xattribute(deferred = true, optional)]
        pub show: Option<Box<super::w3_attributes::Show>>,
        #[xattribute(deferred = true, optional)]
        pub actuate: Option<Box<super::w3_attributes::Actuate>>,
    }
}
#[derive(Debug, ::xmlity::Serialize, ::xmlity::Deserialize, PartialEq, Clone)]
pub enum LinkbaseRef {
    #[xelement(
        name = "linkbaseRef",
        namespace = "http://www.xbrl.org/2003/linkbase",
        allow_unknown_attributes = "any"
    )]
    LinkbaseRef(#[xgroup] linkbase_ref_items::LinkbaseRef),
}

const LINKBASE_REF: &str = r###"
<link:linkbaseRef 
    xmlns:link="http://www.xbrl.org/2003/linkbase" 
    xmlns:xlink="http://www.w3.org/1999/xlink"
    xlink:type="simple" 
    xlink:href="107-01-SchemaContainingALinkbase.xsd#labelLinkbase" 
    xlink:role="http://www.xbrl.org/2003/role/labelLinkbaseRef" 
    xlink:arcrole="http://www.w3.org/1999/xlink/properties/linkbase"/>
"###;

#[test]
fn linkbase_ref() {
    let direct: LinkbaseRef =
        xmlity_quick_xml::from_str(LINKBASE_REF.trim()).expect("Failed to parse linkbaseRef XML");

    let element: xmlity::value::XmlValue =
        xmlity_quick_xml::from_str(LINKBASE_REF.trim()).expect("Failed to parse linkbaseRef XML");

    let indirect: LinkbaseRef =
        xmlity::Deserialize::deserialize(&element).expect("Failed to deserialize linkbaseRef XML");

    assert_eq!(direct, indirect);
}
