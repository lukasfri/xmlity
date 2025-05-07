/// Construct an `XmlValue` from an XML-like literal.
///
/// ```
/// # use xmlity::xml;
/// #
/// let value/*: xmlity::types::value::XmlElement*/ = xml!(
///     <"root">[
///         <"child" "attribute"="value">["Text"]</"child">
///         <![CDATA["CData content"]]>
///         <?"pi-target" "instruction"?>
///         <!-- "Comment" -->
///        "Text node"
///        <"child2":"http://example.com" "attr1"="value1" "attr2"="value2">[]</"child2">
///     ]</"root">
/// );
/// ```
///
/// The macro supports the following XML constructs:
/// - Elements with attributes
/// - Text nodes
/// - CDATA sections
/// - Processing instructions
/// - Comments
/// - Sequences of children
///
/// The macro will automatically choose between an `XmlSeq` and a value-type depending on if the top-level element is a sequence or not.
/// For example, the value above is an `XmlElement` with a sequence of children, while the example below returns an `XmlSeq`:
///
/// ```
/// # use xmlity::xml;
/// #
/// let value/*: xmlity::types::value::XmlSeq<_>*/ = xml!(
///     <"child" "attribute"="value">["Text"]</"child">
///     <"child" "attribute"="value">["Text"]</"child">
/// );
/// ```
#[macro_export]
macro_rules! xml {
    // Hide distracting implementation details from the generated rustdoc.
    ($($xml:tt)+) => {
        $crate::xml_internal!($($xml)+)
    };
}

// Changes are fine as long as `xml_internal!` does not call any new helper
// macros and can still be invoked as `xml_internal!($($xml)+)`.
#[macro_export]
#[doc(hidden)]
macro_rules! xml_internal {
    (@childseq_wrapper [$first_seq_element:expr$(,$($seq_elements:expr),+)?] [$($wrapped_elements:expr,)*]) => {
        $crate::xml_internal!(@childseq_wrapper [$($seq_elements,)*] [$($wrapped_elements,)* $crate::types::value::XmlChild::from($first_seq_element)])
    };
    (@childseq_wrapper [] [$($wrapped_elements:expr,)*]) => {
        $crate::types::value::XmlSeq::from_vec(vec![$($wrapped_elements)*])
    };
    // @seq
    // $unwrapped_if_single = true/false
    // $element_type = @child or @element
    // [$($seq_elements:expr),*] = [$seq_element, ...]
    // $($rest:tt)* = rest of the input

    // Basic types (comments, cdata, pi)
    (@seq $unwrapped_if_single:tt $element_type:tt [$($seq_elements:expr),*] <!--$comment:literal--> $($rest:tt)*) => {
        $crate::xml_internal!(@seq $unwrapped_if_single $element_type [$($seq_elements,)* $crate::types::value::XmlComment::new($comment.as_bytes())] $($rest)*)
    };
    (@seq $unwrapped_if_single:tt $element_type:tt [$($seq_elements:expr),*] <![CDATA[$cdata:literal]]> $($rest:tt)*) => {
        $crate::xml_internal!(@seq $unwrapped_if_single $element_type [$($seq_elements,)* $crate::types::value::XmlCData::new($cdata.as_bytes())] $($rest)*)
    };
    (@seq $unwrapped_if_single:tt $element_type:tt [$($seq_elements:expr),*] <?$target:literal $content:literal?> $($rest:tt)*) => {
        $crate::xml_internal!(@seq $unwrapped_if_single $element_type [$($seq_elements,)* $crate::types::value::XmlProcessingInstruction::new($target.as_bytes(), $content.as_bytes())] $($rest)*)
    };

    // Elements
    // Element adding attribute with namespace
    (@seqelem $unwrapped_if_single:tt $element_type:tt [$($seq_elements:expr),*] $element_name:literal {$element_namespace:expr} [$($attributes:expr),*] $attribute_name:literal:$attribute_namespace:literal = $attribute_value:literal $($rest:tt)*) => {
        $crate::xml_internal!(@seqelem $unwrapped_if_single $element_type [$($seq_elements),*] $element_name {$element_namespace} [$($attributes,)* $crate::types::value::XmlAttribute::new(
            $crate::ExpandedName::new(<$crate::LocalName as std::str::FromStr>::from_str($attribute_name).unwrap(), Some(<$crate::XmlNamespace as std::str::FromStr>::from_str($attribute_namespace).unwrap())),
            $attribute_value
        )] $($rest)*)
    };
    // Element adding attribute without namespace
    (@seqelem $unwrapped_if_single:tt $element_type:tt [$($seq_elements:expr),*] $element_name:literal {$element_namespace:expr} [$($attributes:expr),*] $attribute_name:literal = $attribute_value:literal $($rest:tt)*) => {
        $crate::xml_internal!(@seqelem $unwrapped_if_single $element_type [$($seq_elements),*] $element_name {$element_namespace} [$($attributes,)* $crate::types::value::XmlAttribute::new(
            $crate::ExpandedName::new(<$crate::LocalName as std::str::FromStr>::from_str($attribute_name).unwrap(), $element_namespace),
            $attribute_value
        )] $($rest)*)
    };
    // Element end without children
    (@seqelem $unwrapped_if_single:tt $element_type:tt [$($seq_elements:expr),*] $element_name:literal {$element_namespace:expr} [$($attributes:expr),*] /> $($rest:tt)*) => {
        $crate::xml_internal!(@seq $unwrapped_if_single $element_type [$($seq_elements,)* $crate::types::value::XmlElement::new(
            $crate::ExpandedName::new(<$crate::LocalName as std::str::FromStr>::from_str($element_name).unwrap(), $element_namespace),
        ).with_attributes::<$crate::types::value::XmlAttribute, _>(vec![$($attributes),*])] $($rest)*)
    };
    // Element end with children
    (@seqelem $unwrapped_if_single:tt $element_type:tt [$($seq_elements:expr),*] $element_name:literal {$element_namespace:expr} [$($attributes:expr),*] >[ $($children:tt)* ]</$name2:literal> $($rest:tt)*) => {
        $crate::xml_internal!(@seq $unwrapped_if_single $element_type [$($seq_elements,)* {
            assert_eq!($element_name, $name2, "Starting and ending element names must match");
            $crate::types::value::XmlElement::new(
                $crate::ExpandedName::new(<$crate::LocalName as std::str::FromStr>::from_str($element_name).unwrap(), $element_namespace),
            ).with_attributes::<$crate::types::value::XmlAttribute, _>(vec![$($attributes),*])
            .with_children::<$crate::types::value::XmlChild, _>($crate::xml_internal!(@seq false "child" [] $($children)*))
        }] $($rest)*)
    };

    // Element start
    //With namespace
    (@seq $unwrapped_if_single:tt $element_type:tt [$($seq_elements:expr),*] <$element_name:literal : $element_namespace:literal $($rest:tt)*) => {
        $crate::xml_internal!(@seqelem $unwrapped_if_single $element_type [$($seq_elements),*] $element_name {Some(<$crate::XmlNamespace as std::str::FromStr>::from_str($element_namespace).unwrap())} [] $($rest)*)
    };
    //Without namespace
    (@seq $unwrapped_if_single:tt $element_type:tt [$($seq_elements:expr),*] <$element_name:literal $($rest:tt)*) => {
        $crate::xml_internal!(@seqelem $unwrapped_if_single $element_type [$($seq_elements),*] $element_name {None} [] $($rest)*)
    };

    // Text
    (@seq $unwrapped_if_single:tt $element_type:tt [$($seq_elements:expr),*] $text:literal $($rest:tt)*) => {
        $crate::xml_internal!(@seq $unwrapped_if_single $element_type [$($seq_elements,)* $crate::types::value::XmlText::new($text)] $($rest)*)
    };

    // Sequence ends
    // Ends sequence if single element
    (@seq true $element_type:tt [$seq_element:expr]) => {
        $seq_element
    };
    (@seq $unwrapped_if_single:tt "child" [$($seq_elements:expr),*]) => {
        <$crate::types::value::XmlSeq<$crate::types::value::XmlChild> as FromIterator<$crate::types::value::XmlChild>>::from_iter([$($crate::types::value::XmlChild::from($seq_elements)),*])
    };
    (@seq $unwrapped_if_single:tt "value" [$($seq_elements:expr),*]) => {
        <$crate::types::value::XmlSeq<$crate::types::value::XmlValue> as FromIterator<$crate::types::value::XmlValue>>::from_iter([$($crate::types::value::XmlValue::from($seq_elements)),*])
    };

    // Main entry point for the xml! macro.
    (<!--$comment:literal--> $($rest:tt)*) => {
        $crate::xml_internal!(@seq true "value" [] <!--$comment--> $($rest)*)
    };
    (<?$target:literal $content:literal?> $($rest:tt)*) => {
        $crate::xml_internal!(@seq true "value" [] <?$target $content?> $($rest)*)
    };
    (<![CDATA[$cdata:literal]]> $($rest:tt)*) => {
        $crate::xml_internal!(@seq true "value" [] <![CDATA[$cdata]]> $($rest)*)
    };
    (<$element_name:literal : $namespace:literal $($rest:tt)*) => {
        $crate::xml_internal!(@seq true "value" [] <$element_name : $namespace $($rest)*)
    };
    (<$element_name:literal $($rest:tt)*) => {
        $crate::xml_internal!(@seq true "value" [] <$element_name $($rest)*)
    };
    ($text:literal $($rest:tt)*) => {
        $crate::xml_internal!(@seq true "value" [] $text $($rest)*)
    };
}
