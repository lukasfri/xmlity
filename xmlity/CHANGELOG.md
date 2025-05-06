# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.3](https://github.com/lukasfri/xmlity/compare/xmlity-v0.0.2...xmlity-v0.0.3) - 2025-05-06

### Added

- *(core,quick-xml)* [**breaking**] Changes element serialization API to finish serializing name before attributes ([#62](https://github.com/lukasfri/xmlity/pull/62))
- *(core)* Adds implementations for `std` datatypes ([#58](https://github.com/lukasfri/xmlity/pull/58))
- *(core)* Adds `into_parts` methods to split `ExpandedName` and `QName`. ([#61](https://github.com/lukasfri/xmlity/pull/61))

## [0.0.2](https://github.com/lukasfri/xmlity/compare/xmlity-v0.0.1...xmlity-v0.0.2) - 2025-05-02

### Added

- *(core)* [**breaking**] Removed `SerializeChildren` in favour of `SerializeSeq`. ([#47](https://github.com/lukasfri/xmlity/pull/47))

### Fixed

- *(core)* Changed XmlRoot to serialize correctly as sequence. ([#28](https://github.com/lukasfri/xmlity/pull/28))
- *(core)* [**breaking**] Fixes incorrect function signatures for "dangerous" methods. ([#27](https://github.com/lukasfri/xmlity/pull/27))

### Other

- Improved docs bringing documentation up to date for release 0.0.2 ([#48](https://github.com/lukasfri/xmlity/pull/48))
- Removed accidental serde mention from license notice ([#46](https://github.com/lukasfri/xmlity/pull/46))

## [0.0.1](https://github.com/lukasfri/xmlity/compare/xmlity-v0.0.0...xmlity-v0.0.1) - 2025-04-16

### Added

- *(docs)* Improved documentation on derive macros. ([#7](https://github.com/lukasfri/xmlity/pull/7))
- *(types)* [**breaking**] Reworks XmlRoot to be more correct ([#14](https://github.com/lukasfri/xmlity/pull/14))
- *(derive)* [**breaking**] Add derive for (non-element) structs to be serialized as a sequence ([#18](https://github.com/lukasfri/xmlity/pull/18))
- *(core)* Add serialize impl for bool and (de)serialize for Box<T> ([#15](https://github.com/lukasfri/xmlity/pull/15))
- *(core)* Adds more implementations for Rust primitives. ([#13](https://github.com/lukasfri/xmlity/pull/13))
- *(docs)* READMEs: Improvements to main, added missing and added tests ([#6](https://github.com/lukasfri/xmlity/pull/6))

### Other

## [0.0.0](https://github.com/lukasfri/xmlity/releases/tag/xmlity-v0.0.0) - 2025-04-09

### Other

- Initial commit
