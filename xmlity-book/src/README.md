# Introduction to XMLity

XMLity is a (de)serialization library for XML, inspired by [`serde`][serde] and improves upon XML (de)serialization libraries such as [`yaserde`][yaserde] and [`quick-xml`][quick-xml] by providing a more flexible API that is more powerful, utilising primairly a trial and error approach to parsing XML.

[serde]: https://serde.rs/
[yaserde]: https://github.com/media-io/yaserde
[quick-xml]: https://github.com/tafia/quick-xml
[xml-rs]: https://github.com/kornelski/xml-rs

Want to get started? [Click here](./1_getting_started/README.md).

## Comparison

Why does XMLity need to exist? Doesn't other XML-parsers/deserializers exist in Rust and why should I use/contribute to XMLity instead? To answer these questions, we've provided a feature chart below:

<sup>
If something stated here is wrong, please submit a PR! It is not guaranteed to be correct, but should be a pretty fair description. The list tries to rank the features in order of importance, but it is ofcourse subjective.
</sup>

| Feature                                                                      | XMLity              | [Yaserde][yaserde] | [`quick-xml`][quick-xml] <br> <sup>(`serde` feature)</sup> |
| ---------------------------------------------------------------------------- | ------------------- | ------------------ | ---------------------------------------------------------- |
| Namespace support                                                            | Yes                 | Yes                | No                                                         |
| Enum string value                                                            | Yes                 | Yes                | Yes                                                        |
| Enum number value                                                            | No<sup>?</sup>      | Yes                | Yes                                                        |
| Trial and error deserialization                                              | Yes                 | No                 | No <sup>\*</sup>                                           |
| Standalone named elements <br> <sup>(inherit element names from types)</sup> | Yes                 | No                 | No                                                         |
| Order-based deserialization                                                  | Yes                 | No                 | Yes                                                        |
| XSD Generator                                                                | No <sup>+</sup>     | Yes                | Yes                                                        |
| Multiple reader/writer implementation support                                | Yes <sup>\*\*</sup> | No                 | No                                                         |

<strong>\+</strong> &ensp; Being worked on.

<strong>?</strong> &ensp; Planned.

<strong>\*</strong> &ensp; While `quick-xml` has partial support, it is insufficient for any range of possible values. It only supports textual nodes, meaning trial and error deserialization is not possible for proper elements.

<strong>\*\*</strong> &nbsp; While XMLity does have support for multiple readers/writers, the only current one that exists is the official one using `quick-xml`, even if others are supported due to the data-model being completely deconnected from any other library. This should change in the near future as support for [`xml-rs`](https://crates.io/crates/xml-rs) is planned.
