# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [2.0.2] - 2018-12-29
### Fixed
- Updated dependencies

## [2.0.1] - 2017-12-31
### Fixed
- Updated `README.md` to conform to the new API.

## [2.0.0] - 2017-12-31
### Added
- This changelog (`CHANGELOG.md`).

### Changed
- Updated `log` to 0.4.0, which forced some API changes.
- Reworked docs.
- The thread ID is now printed in hex to reduce clutter and make visual
  identification easier.

### Removed
- `SimpleLogger` struct from the public API.
- [`Sync`](https://doc.rust-lang.org/std/marker/trait.Sync.html) constraint on
  custom sinks.

## [1.0.1] 2017-07-13
### Fixed
- Links and Travis build. No functional changes.

## [1.0.0] 2017-07-13
### Added
- Initial release
