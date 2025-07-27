# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.8](https://github.com/lukasfri/xmlity/compare/xmlity-quick-xml-v0.0.7...xmlity-quick-xml-v0.0.8) - 2025-07-27

### Added

- *(derive)* Adds `ignore_comments` option to easily ignore comments in elements and values. ([#120](https://github.com/lukasfri/xmlity/pull/120))
- Adds external data support to deserialization process. ([#116](https://github.com/lukasfri/xmlity/pull/116))
- *(derive)* Adds `serialize_with`/`deserialize_with`/`with` for easy `(De)serialize` implementations. ([#109](https://github.com/lukasfri/xmlity/pull/109))

### Fixed

- *(value)* Fix stack overflow for subvalue in struct from value. ([#125](https://github.com/lukasfri/xmlity/pull/125))
- *(value)* Fixed `deserialize_seq` on most XML value types. ([#124](https://github.com/lukasfri/xmlity/pull/124))
- *(quick-xml)* Fix bug in sub-access of attributes in elements. ([#123](https://github.com/lukasfri/xmlity/pull/123))
- *(core)* [**breaking**] Change `SerializeSeq` trait to return `()` for each individual element. ([#115](https://github.com/lukasfri/xmlity/pull/115))
- *(quick-xml)* Fixed sequence-based text values (enums) not being supported in attributes. ([#114](https://github.com/lukasfri/xmlity/pull/114))
- *(derive)* Support serialization of enums with no arms. ([#113](https://github.com/lukasfri/xmlity/pull/113))

### Other

- *(derive)* Adds test for `extendable = "iterator"`  option. ([#119](https://github.com/lukasfri/xmlity/pull/119))

## [0.0.7](https://github.com/lukasfri/xmlity/compare/xmlity-quick-xml-v0.0.6...xmlity-quick-xml-v0.0.7) - 2025-06-23

### Added

- *(derive)* [**breaking**] Rename `loose` value order to `strict`. ([#103](https://github.com/lukasfri/xmlity/pull/103))

### Fixed

- *(quick-xml)* Fixed `xmlity-quick-xml` `Deserializer` to correctly handle sequences. ([#106](https://github.com/lukasfri/xmlity/pull/106))
- Fixes invalid deserialization of a combination of values ([#102](https://github.com/lukasfri/xmlity/pull/102))
- Fixed regression in values with group elements. ([#99](https://github.com/lukasfri/xmlity/pull/99))
- *(derive)* Fixes regression of inline declarations of elements with group child not compiling. ([#96](https://github.com/lukasfri/xmlity/pull/96))
- Cleanup to pass `cargo clippy`. ([#98](https://github.com/lukasfri/xmlity/pull/98))
- Fixes recursively empty values not deserializing correctly. ([#94](https://github.com/lukasfri/xmlity/pull/94))

## [0.0.6](https://github.com/lukasfri/xmlity/compare/xmlity-quick-xml-v0.0.5...xmlity-quick-xml-v0.0.6) - 2025-06-04

### Added

- *(derive)* Adds `(De)SerializationGroup` support for unit groups. ([#93](https://github.com/lukasfri/xmlity/pull/93))
- Adds `NamespaceContext::default_namespace` that gives access to default namespace. ([#90](https://github.com/lukasfri/xmlity/pull/90))
- *(derive)* [**breaking**] Made item structs work like element children. Improved IgnoreWhitespace option with tests. ([#88](https://github.com/lukasfri/xmlity/pull/88))

### Fixed

- *(derive)* Fixes `(De)SerializationGroup` support for unnamed field groups. ([#92](https://github.com/lukasfri/xmlity/pull/92))
- Changed `Option<T>` to work correctly using trial-and-error deserialization. ([#91](https://github.com/lukasfri/xmlity/pull/91))
- *(quick-xml)* Fixes bug involving attributes not resolving namespaces correctly. ([#89](https://github.com/lukasfri/xmlity/pull/89))

## [0.0.5](https://github.com/lukasfri/xmlity/compare/xmlity-quick-xml-v0.0.4...xmlity-quick-xml-v0.0.5) - 2025-05-30

### Added

- *(derive)* [**breaking**] Adds options for inline declarations, conditional serialization, updates existing options. ([#82](https://github.com/lukasfri/xmlity/pull/82))
- *(core)* [**breaking**] Changes attributes to use XMLity values instead of strings. ([#83](https://github.com/lukasfri/xmlity/pull/83))

## [0.0.4](https://github.com/lukasfri/xmlity/compare/xmlity-quick-xml-v0.0.3...xmlity-quick-xml-v0.0.4) - 2025-05-09

### Added

- _(core)_ [**breaking**] Refactored XmlValue, made it a top-level export. ([#75](https://github.com/lukasfri/xmlity/pull/75))
- _(derive,quick-xml)_ [**breaking**] Rework data access to be lifetime-dependent. Adds whitespace configuration. ([#73](https://github.com/lukasfri/xmlity/pull/73))
- _(core)_ [**breaking**] Reworked content access and namespace access during deserialization ([#72](https://github.com/lukasfri/xmlity/pull/72))

### Other

- _(quick-xml)_ Added `yaserde` to benchmarks. ([#70](https://github.com/lukasfri/xmlity/pull/70))
- _(quick-xml)_ Optimizations to `quick-xml` Deserializer. ([#67](https://github.com/lukasfri/xmlity/pull/67))

## [0.0.3](https://github.com/lukasfri/xmlity/compare/xmlity-quick-xml-v0.0.2...xmlity-quick-xml-v0.0.3) - 2025-05-06

### Added

- _(quick-xml)_ [**breaking**] Cleanup of the API of `xmlity-quick-xml`. ([#64](https://github.com/lukasfri/xmlity/pull/64))
- _(core,quick-xml)_ [**breaking**] Changes element serialization API to finish serializing name before attributes ([#62](https://github.com/lukasfri/xmlity/pull/62))
- _(derive)_ Changed structs and enum variants to use common (de)serialization logic ([#54](https://github.com/lukasfri/xmlity/pull/54))

### Other

- _(quick-xml)_ Adds benchmarks for quick-xml (De)serializer ([#60](https://github.com/lukasfri/xmlity/pull/60))

## [0.0.2](https://github.com/lukasfri/xmlity/compare/xmlity-quick-xml-v0.0.1...xmlity-quick-xml-v0.0.2) - 2025-05-02

### Added

- _(core)_ [**breaking**] Removed `SerializeChildren` in favour of `SerializeSeq`. ([#47](https://github.com/lukasfri/xmlity/pull/47))
- _(derive)_ [**breaking**] Local element/attribute declarations ([#41](https://github.com/lukasfri/xmlity/pull/41))
- _(derive)_ Support both values and trial-and-error in enums ([#35](https://github.com/lukasfri/xmlity/pull/35))
- _(quick-xml)_ Added `to_string_pretty` that supports indentation. ([#23](https://github.com/lukasfri/xmlity/pull/23))

### Fixed

- _(quick-xml)_ If zero children are serialized, serialize as empty. ([#30](https://github.com/lukasfri/xmlity/pull/30))
- _(quick-xml)_ Don't escape XML incorrectly. ([#36](https://github.com/lukasfri/xmlity/pull/36))
- _(derive)_ [**breaking**] Fixed xvalue in SerializationGroup/DeserializationGroup. ([#25](https://github.com/lukasfri/xmlity/pull/25))
- _(quick-xml)_ Adds top-level scope including xml prefix. ([#26](https://github.com/lukasfri/xmlity/pull/26))

### Other

- _(tests)_ Restructure tests to be simpler and organized by type ([#40](https://github.com/lukasfri/xmlity/pull/40))

## [0.0.1](https://github.com/lukasfri/xmlity/compare/xmlity-quick-xml-v0.0.0...xmlity-quick-xml-v0.0.1) - 2025-04-16

### Added

- _(derive)_ [**breaking**] Implement deserialize option "extendable". ([#20](https://github.com/lukasfri/xmlity/pull/20))
- _(types)_ [**breaking**] Reworks XmlRoot to be more correct ([#14](https://github.com/lukasfri/xmlity/pull/14))
- _(derive)_ Adds ability to choose namespace by path instead of text. ([#12](https://github.com/lukasfri/xmlity/pull/12))
- _(derive)_ Add type generic support for elements and groups. ([#16](https://github.com/lukasfri/xmlity/pull/16))
- _(test)_ Added tests for renaming enum values. ([#9](https://github.com/lukasfri/xmlity/pull/9))
- _(docs)_ READMEs: Improvements to main, added missing and added tests ([#6](https://github.com/lukasfri/xmlity/pull/6))

### Fixed

- _(derive)_ Fixed broken group in group derive. ([#10](https://github.com/lukasfri/xmlity/pull/10))
- _(quick-xml)_ Fixed sub-sub access of element attributes. ([#11](https://github.com/lukasfri/xmlity/pull/11))

### Other

## [0.0.0](https://github.com/lukasfri/xmlity/releases/tag/xmlity-quick-xml-v0.0.0) - 2025-04-09

### Other

- Initial commit
