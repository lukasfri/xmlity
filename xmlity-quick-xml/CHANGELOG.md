# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.3](https://github.com/lukasfri/xmlity/compare/xmlity-quick-xml-v0.0.2...xmlity-quick-xml-v0.0.3) - 2025-05-06

### Added

- *(quick-xml)* [**breaking**] Cleanup of the API of `xmlity-quick-xml`. ([#64](https://github.com/lukasfri/xmlity/pull/64))
- *(core,quick-xml)* [**breaking**] Changes element serialization API to finish serializing name before attributes ([#62](https://github.com/lukasfri/xmlity/pull/62))
- *(derive)* Changed structs and enum variants to use common (de)serialization logic ([#54](https://github.com/lukasfri/xmlity/pull/54))

### Other

- *(quick-xml)* Adds benchmarks for quick-xml (De)serializer ([#60](https://github.com/lukasfri/xmlity/pull/60))

## [0.0.2](https://github.com/lukasfri/xmlity/compare/xmlity-quick-xml-v0.0.1...xmlity-quick-xml-v0.0.2) - 2025-05-02

### Added

- *(core)* [**breaking**] Removed `SerializeChildren` in favour of `SerializeSeq`. ([#47](https://github.com/lukasfri/xmlity/pull/47))
- *(derive)* [**breaking**] Local element/attribute declarations ([#41](https://github.com/lukasfri/xmlity/pull/41))
- *(derive)* Support both values and trial-and-error in enums ([#35](https://github.com/lukasfri/xmlity/pull/35))
- *(quick-xml)* Added `to_string_pretty` that supports indentation. ([#23](https://github.com/lukasfri/xmlity/pull/23))

### Fixed

- *(quick-xml)* If zero children are serialized, serialize as empty. ([#30](https://github.com/lukasfri/xmlity/pull/30))
- *(quick-xml)* Don't escape XML incorrectly. ([#36](https://github.com/lukasfri/xmlity/pull/36))
- *(derive)* [**breaking**] Fixed xvalue in SerializationGroup/DeserializationGroup. ([#25](https://github.com/lukasfri/xmlity/pull/25))
- *(quick-xml)* Adds top-level scope including xml prefix. ([#26](https://github.com/lukasfri/xmlity/pull/26))

### Other

- *(tests)* Restructure tests to be simpler and organized by type ([#40](https://github.com/lukasfri/xmlity/pull/40))

## [0.0.1](https://github.com/lukasfri/xmlity/compare/xmlity-quick-xml-v0.0.0...xmlity-quick-xml-v0.0.1) - 2025-04-16

### Added

- *(derive)* [**breaking**] Implement deserialize option "extendable". ([#20](https://github.com/lukasfri/xmlity/pull/20))
- *(types)* [**breaking**] Reworks XmlRoot to be more correct ([#14](https://github.com/lukasfri/xmlity/pull/14))
- *(derive)* Adds ability to choose namespace by path instead of text. ([#12](https://github.com/lukasfri/xmlity/pull/12))
- *(derive)* Add type generic support for elements and groups. ([#16](https://github.com/lukasfri/xmlity/pull/16))
- *(test)* Added tests for renaming enum values. ([#9](https://github.com/lukasfri/xmlity/pull/9))
- *(docs)* READMEs: Improvements to main, added missing and added tests ([#6](https://github.com/lukasfri/xmlity/pull/6))

### Fixed

- *(derive)* Fixed broken group in group derive. ([#10](https://github.com/lukasfri/xmlity/pull/10))
- *(quick-xml)* Fixed sub-sub access of element attributes. ([#11](https://github.com/lukasfri/xmlity/pull/11))

### Other

## [0.0.0](https://github.com/lukasfri/xmlity/releases/tag/xmlity-quick-xml-v0.0.0) - 2025-04-09

### Other

- Initial commit
