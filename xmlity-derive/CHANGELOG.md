# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.6](https://github.com/lukasfri/xmlity/compare/xmlity-derive-v0.0.5...xmlity-derive-v0.0.6) - 2025-06-04

### Added

- *(derive)* Adds `(De)SerializationGroup` support for unit groups. ([#93](https://github.com/lukasfri/xmlity/pull/93))
- *(derive)* [**breaking**] Made item structs work like element children. Improved IgnoreWhitespace option with tests. ([#88](https://github.com/lukasfri/xmlity/pull/88))

### Fixed

- *(derive)* Fixes `(De)SerializationGroup` support for unnamed field groups. ([#92](https://github.com/lukasfri/xmlity/pull/92))

## [0.0.5](https://github.com/lukasfri/xmlity/compare/xmlity-derive-v0.0.4...xmlity-derive-v0.0.5) - 2025-05-30

### Added

- *(derive)* [**breaking**] Adds options for inline declarations, conditional serialization, updates existing options. ([#82](https://github.com/lukasfri/xmlity/pull/82))
- *(core)* [**breaking**] Changes attributes to use XMLity values instead of strings. ([#83](https://github.com/lukasfri/xmlity/pull/83))

## [0.0.4](https://github.com/lukasfri/xmlity/compare/xmlity-derive-v0.0.3...xmlity-derive-v0.0.4) - 2025-05-09

### Added

- *(derive,quick-xml)* [**breaking**] Rework data access to be lifetime-dependent. Adds whitespace configuration. ([#73](https://github.com/lukasfri/xmlity/pull/73))

## [0.0.3](https://github.com/lukasfri/xmlity/compare/xmlity-derive-v0.0.2...xmlity-derive-v0.0.3) - 2025-05-06

### Added

- *(core,quick-xml)* [**breaking**] Changes element serialization API to finish serializing name before attributes ([#62](https://github.com/lukasfri/xmlity/pull/62))
- *(derive)* Changed structs and enum variants to use common (de)serialization logic ([#54](https://github.com/lukasfri/xmlity/pull/54))

### Other

- *(derive)* Updates docs to provide details on new unified enum API. ([#65](https://github.com/lukasfri/xmlity/pull/65))

## [0.0.2](https://github.com/lukasfri/xmlity/compare/xmlity-derive-v0.0.1...xmlity-derive-v0.0.2) - 2025-05-02

### Added

- *(core)* [**breaking**] Removed `SerializeChildren` in favour of `SerializeSeq`. ([#47](https://github.com/lukasfri/xmlity/pull/47))
- *(derive)* [**breaking**] Local element/attribute declarations ([#41](https://github.com/lukasfri/xmlity/pull/41))
- *(derive)* Support both values and trial-and-error in enums ([#35](https://github.com/lukasfri/xmlity/pull/35))

### Fixed

- *(derive)* Make more type parameters odd to remove type conflicts ([#43](https://github.com/lukasfri/xmlity/pull/43))
- *(derive)* Use a more odd deserializer type to not interfere with structs ([#42](https://github.com/lukasfri/xmlity/pull/42))
- *(derive)* [**breaking**] Fixed xvalue in SerializationGroup/DeserializationGroup. ([#25](https://github.com/lukasfri/xmlity/pull/25))

### Other

- Improved docs bringing documentation up to date for release 0.0.2 ([#48](https://github.com/lukasfri/xmlity/pull/48))
- *(derive)* Refactored derive to have docs from .md files and moved code. ([#49](https://github.com/lukasfri/xmlity/pull/49))

## [0.0.1](https://github.com/lukasfri/xmlity/compare/xmlity-derive-v0.0.0...xmlity-derive-v0.0.1) - 2025-04-16

### Added

- *(docs)* Improved documentation on derive macros. ([#7](https://github.com/lukasfri/xmlity/pull/7))
- *(derive)* [**breaking**] Cleaned up errors and made attribute options exclusive when they don't work together ([#21](https://github.com/lukasfri/xmlity/pull/21))
- *(derive)* [**breaking**] Implement deserialize option "extendable". ([#20](https://github.com/lukasfri/xmlity/pull/20))
- *(derive)* Adds ability to choose namespace by path instead of text. ([#12](https://github.com/lukasfri/xmlity/pull/12))
- *(derive)* [**breaking**] Add derive for (non-element) structs to be serialized as a sequence ([#18](https://github.com/lukasfri/xmlity/pull/18))
- *(derive)* Add type generic support for elements and groups. ([#16](https://github.com/lukasfri/xmlity/pull/16))
- *(docs)* READMEs: Improvements to main, added missing and added tests ([#6](https://github.com/lukasfri/xmlity/pull/6))

### Fixed

- *(derive)* Fixed broken group in group derive. ([#10](https://github.com/lukasfri/xmlity/pull/10))
- *(derive)* Disambiguate associated types in derive macros. ([#8](https://github.com/lukasfri/xmlity/pull/8))

### Other

## [0.0.0](https://github.com/lukasfri/xmlity/releases/tag/xmlity-derive-v0.0.0) - 2025-04-09

### Other

- Initial commit
