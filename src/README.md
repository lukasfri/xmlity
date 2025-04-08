# Introduction

## Comparison

Why does Xmlity need to exist? Doesn't other XML-parsers/deserializers exist in Rust and why should I use/contribute to Xmlity instead? To answer these questions, we've provided a feature chart below that

| Feature                                       | Xmlity | Yaserde | quick-xml (serde) |
| --------------------------------------------- | ------ | ------- | ----------------- |
| Trial and error deserialization               | ☑      | ☐       | ☑                 |
| Multiple reader/writer implementation support | ☑ \*   | ☐       | ☐                 |

\* While Xmlity does have support for multiple readers/writers, the only current one that exists is one using quick-xml, even if others are supported due to the data-model being completely deconnected from any other library. This should change in the near future.
