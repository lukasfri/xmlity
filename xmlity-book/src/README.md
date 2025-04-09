# Introduction to XMLity

XMLity is a (de)serialization library for XML, inspired by [Serde](https://serde.rs/) and improves upon XML (de)serialization libraries such as [yaserde](https://github.com/media-io/yaserde) and [quick-xml](https://github.com/tafia/quick-xml) by providing a more flexible API that is more powerful, utilising primairly a trial and error approach to parsing XML. This can inherently be a bit slower than other libraries, but it allows for more complex XML structures to be parsed.

Want to get started? [Click here](./1_getting_started/README.md).

## Comparison

Why does XMLity need to exist? Doesn't other XML-parsers/deserializers exist in Rust and why should I use/contribute to XMLity instead? To answer these questions, we've provided a feature chart below that

| Feature                                       | XMLity | Yaserde | quick-xml (serde) |
| --------------------------------------------- | ------ | ------- | ----------------- |
| Trial and error deserialization               | Yes    | No      | Yes               |
| Multiple reader/writer implementation support | Yes \* | No      | Yes               |

\* While XMLity does have support for multiple readers/writers, the only current one that exists is one using quick-xml, even if others are supported due to the data-model being completely deconnected from any other library. This should change in the near future.
