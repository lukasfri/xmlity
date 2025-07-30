use crate::define_serialize_test;
use xmlity::{Serialize, SerializeAttribute};

/// Test struct with enforce_prefix = false (default behavior)
#[derive(Debug, PartialEq, Serialize)]
#[xelement(
    name = "test",
    namespace = "http://example.com/test",
    preferred_prefix = "pref"
)]
pub struct TestElementNoEnforce {
    value: String,
}

/// Test struct with enforce_prefix = true
#[derive(Debug, PartialEq, Serialize)]
#[xelement(
    name = "test",
    namespace = "http://example.com/test",
    preferred_prefix = "pref",
    enforce_prefix = true
)]
pub struct TestElementEnforcePrefix {
    value: String,
}

/// Test struct for attribute serialization with enforce_prefix = false
#[derive(Debug, PartialEq, SerializeAttribute)]
#[xattribute(
    name = "testattr",
    namespace = "http://example.com/test",
    preferred_prefix = "pref"
)]
pub struct TestAttributeNoEnforce(String);

/// Test struct for attribute serialization with enforce_prefix = true
#[derive(Debug, PartialEq, SerializeAttribute)]
#[xattribute(
    name = "testattr",
    namespace = "http://example.com/test",
    preferred_prefix = "pref",
    enforce_prefix = true
)]
pub struct TestAttributeEnforcePrefix(String);

#[derive(Debug, PartialEq, Serialize)]
#[xelement(
    name = "wrapper",
    namespace = "http://example.com/test",
    preferred_prefix = "wrapper-ns"
)]
struct AttrWrapper<TAttr: SerializeAttribute> {
    #[xattribute(deferred)]
    attr: TAttr,
}
#[derive(Debug, PartialEq, Serialize)]
#[xelement(
    name = "wrapper",
    namespace = "http://example.com/irrelevant",
    preferred_prefix = "wrapper-ns"
)]
struct AttrIrrelevantWrapper<TAttr: SerializeAttribute> {
    #[xattribute(deferred)]
    attr: TAttr,
}

#[derive(Debug, PartialEq, Serialize)]
#[xelement(
    name = "wrapper",
    namespace = "http://example.com/test",
    preferred_prefix = "wrapper-ns"
)]
struct ValueWrapper<T: Serialize> {
    attr: T,
}

define_serialize_test!(
    test_element_no_enforce,
    [
        (
            TestElementNoEnforce {
                value: "test_value".to_string()
            },
            r#"<pref:test xmlns:pref="http://example.com/test">test_value</pref:test>"#
        ),
        (
            TestElementEnforcePrefix {
                value: "test_value".to_string()
            },
            r#"<pref:test xmlns:pref="http://example.com/test">test_value</pref:test>"#
        ),
        (
            AttrIrrelevantWrapper {
                attr: TestAttributeNoEnforce("attribute_value".to_string())
            },
            r#"<wrapper-ns:wrapper xmlns:wrapper-ns="http://example.com/irrelevant" xmlns:pref="http://example.com/test" pref:testattr="attribute_value"/>"#
        ),
        (
            AttrIrrelevantWrapper {
                attr: TestAttributeEnforcePrefix("attribute_value".to_string())
            },
            r#"<wrapper-ns:wrapper xmlns:wrapper-ns="http://example.com/irrelevant" xmlns:pref="http://example.com/test" pref:testattr="attribute_value"/>"#
        ),
        (
            ValueWrapper {
                attr: TestElementNoEnforce {
                    value: "test_value".to_string()
                }
            },
            r#"<wrapper-ns:wrapper xmlns:wrapper-ns="http://example.com/test"><wrapper-ns:test>test_value</wrapper-ns:test></wrapper-ns:wrapper>"#
        ),
        (
            ValueWrapper {
                attr: TestElementEnforcePrefix {
                    value: "test_value".to_string()
                }
            },
            r#"<wrapper-ns:wrapper xmlns:wrapper-ns="http://example.com/test"><pref:test xmlns:pref="http://example.com/test">test_value</pref:test></wrapper-ns:wrapper>"#
        ),
        (
            AttrWrapper {
                attr: TestAttributeNoEnforce("attribute_value".to_string())
            },
            r#"<wrapper-ns:wrapper xmlns:wrapper-ns="http://example.com/test" wrapper-ns:testattr="attribute_value"/>"#
        ),
        (
            AttrWrapper {
                attr: TestAttributeEnforcePrefix("attribute_value".to_string())
            },
            r#"<wrapper-ns:wrapper xmlns:wrapper-ns="http://example.com/test" xmlns:pref="http://example.com/test" pref:testattr="attribute_value"/>"#
        )
    ]
);
