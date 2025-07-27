# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.8](https://github.com/lukasfri/xmlity/compare/xmlity-v0.0.7...xmlity-v0.0.8) - 2025-07-27

### Added

- *(core)* Infallible error types. ([#122](https://github.com/lukasfri/xmlity/pull/122))
- *(value)* Make visitors of XML value types public. ([#117](https://github.com/lukasfri/xmlity/pull/117))
- Adds external data support to deserialization process. ([#116](https://github.com/lukasfri/xmlity/pull/116))
- *(core)* Add `(De)serialize` impl for `isize`/`usize` non-zero types. ([#112](https://github.com/lukasfri/xmlity/pull/112))
- *(core)* Add `(De)serialize` impls for non-zero primitives. ([#111](https://github.com/lukasfri/xmlity/pull/111))

### Fixed

- *(value)* Fix stack overflow for subvalue in struct from value. ([#125](https://github.com/lukasfri/xmlity/pull/125))
- *(value)* Fixed `deserialize_seq` on most XML value types. ([#124](https://github.com/lukasfri/xmlity/pull/124))
- *(core)* [**breaking**] Change `SerializeSeq` trait to return `()` for each individual element. ([#115](https://github.com/lukasfri/xmlity/pull/115))

## [0.0.7](https://github.com/lukasfri/xmlity/compare/xmlity-v0.0.6...xmlity-v0.0.7) - 2025-06-23

### Added

- *(core)* [**breaking**] Adds `(De)serializationGroup` for `()` and cleans up implementation on `Box<T>`. ([#100](https://github.com/lukasfri/xmlity/pull/100))

### Fixed

- Fixes recursively empty values not deserializing correctly. ([#94](https://github.com/lukasfri/xmlity/pull/94))

## [0.0.6](https://github.com/lukasfri/xmlity/compare/xmlity-v0.0.5...xmlity-v0.0.6) - 2025-06-04

### Added

- Adds `NamespaceContext::default_namespace` that gives access to default namespace. ([#90](https://github.com/lukasfri/xmlity/pull/90))
- *(core)* Adds XmlSchema XmlNamespace const. ([#87](https://github.com/lukasfri/xmlity/pull/87))
- *(core)* Implements Serialize and Deserialize for LocalName, Prefix and XmlNamespace. ([#84](https://github.com/lukasfri/xmlity/pull/84))

### Fixed

- Changed `Option<T>` to work correctly using trial-and-error deserialization. ([#91](https://github.com/lukasfri/xmlity/pull/91))
- *(core)* Fixed lifetime on Deserialize impl of LocalName and Prefix. ([#86](https://github.com/lukasfri/xmlity/pull/86))

## [0.0.5](https://github.com/lukasfri/xmlity/compare/xmlity-v0.0.4...xmlity-v0.0.5) - 2025-05-30

### Added

- *(core)* [**breaking**] Changes attributes to use XMLity values instead of strings. ([#83](https://github.com/lukasfri/xmlity/pull/83))
- *(core)* Added missing `Display` trait to `XmlNamespace`. ([#79](https://github.com/lukasfri/xmlity/pull/79))

## [0.0.4](https://github.com/lukasfri/xmlity/compare/xmlity-v0.0.3...xmlity-v0.0.4) - 2025-05-09

### Added

- _(core)_ [**breaking**] Refactored XmlValue, made it a top-level export. ([#75](https://github.com/lukasfri/xmlity/pull/75))
- _(derive,quick-xml)_ [**breaking**] Rework data access to be lifetime-dependent. Adds whitespace configuration. ([#73](https://github.com/lukasfri/xmlity/pull/73))
- _(core)_ [**breaking**] Reworked content access and namespace access during deserialization ([#72](https://github.com/lukasfri/xmlity/pull/72))
- _(core)_ Improvements to API including `as_ref` and namespace consts ([#66](https://github.com/lukasfri/xmlity/pull/66))

## [0.0.3](https://github.com/lukasfri/xmlity/compare/xmlity-v0.0.2...xmlity-v0.0.3) - 2025-05-06

### Added

- _(core,quick-xml)_ [**breaking**] Changes element serialization API to finish serializing name before attributes ([#62](https://github.com/lukasfri/xmlity/pull/62))
- _(core)_ Adds implementations for `std` datatypes ([#58](https://github.com/lukasfri/xmlity/pull/58))
- _(core)_ Adds `into_parts` methods to split `ExpandedName` and `QName`. ([#61](https://github.com/lukasfri/xmlity/pull/61))

## [0.0.2](https://github.com/lukasfri/xmlity/compare/xmlity-v0.0.1...xmlity-v0.0.2) - 2025-05-02

### Added

- _(core)_ [**breaking**] Removed `SerializeChildren` in favour of `SerializeSeq`. ([#47](https://github.com/lukasfri/xmlity/pull/47))

### Fixed

- _(core)_ Changed XmlRoot to serialize correctly as sequence. ([#28](https://github.com/lukasfri/xmlity/pull/28))
- _(core)_ [**breaking**] Fixes incorrect function signatures for "dangerous" methods. ([#27](https://github.com/lukasfri/xmlity/pull/27))

### Other

- Improved docs bringing documentation up to date for release 0.0.2 ([#48](https://github.com/lukasfri/xmlity/pull/48))
- Removed accidental serde mention from license notice ([#46](https://github.com/lukasfri/xmlity/pull/46))

## [0.0.1](https://github.com/lukasfri/xmlity/compare/xmlity-v0.0.0...xmlity-v0.0.1) - 2025-04-16

### Added

- _(docs)_ Improved documentation on derive macros. ([#7](https://github.com/lukasfri/xmlity/pull/7))
- _(types)_ [**breaking**] Reworks XmlRoot to be more correct ([#14](https://github.com/lukasfri/xmlity/pull/14))
- _(derive)_ [**breaking**] Add derive for (non-element) structs to be serialized as a sequence ([#18](https://github.com/lukasfri/xmlity/pull/18))
- _(core)_ Add serialize impl for bool and (de)serialize for Box<T> ([#15](https://github.com/lukasfri/xmlity/pull/15))
- _(core)_ Adds more implementations for Rust primitives. ([#13](https://github.com/lukasfri/xmlity/pull/13))
- _(docs)_ READMEs: Improvements to main, added missing and added tests ([#6](https://github.com/lukasfri/xmlity/pull/6))

### Other

## [0.0.0](https://github.com/lukasfri/xmlity/releases/tag/xmlity-v0.0.0) - 2025-04-09

### Other

- Initial commit
