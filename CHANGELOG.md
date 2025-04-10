# Changelog

## v0.1.0

Initial release to crates.io.

## v0.2.0

### Changed (**breaking**)
- Project layout now is a workspace with two packages `haitaka_types` and `haitaka`. This was necessary in order to write the 'haitaka` build script that creates sliding moves tables at build time. The build script relies on `haitaka_types`. This layour now mirrors the `cozy-chess` layout.

### Added
- Support for slider move creation by Magic Bitboards. This is now the default implementation.

### Fixed
- Fixed several bugs in move creation. See git log for details.
