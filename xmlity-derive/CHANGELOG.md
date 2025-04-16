# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

- release v0.0.0

## [0.0.0](https://github.com/lukasfri/xmlity/releases/tag/xmlity-derive-v0.0.0) - 2025-04-09

### Other

- Initial commit
