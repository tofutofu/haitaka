# Changelog

## v0.2.2 (Unreleased)
- Fixed a bug in the generation of Pawn drops (if the first candidate drop was an 
illegal mate, then all Pawn drops would be skipped).
- Fixed a bug in calculating checkers and pins after playing a move (checks with lance were
not updated).

## v0.2.1

- Fixed SFEN display of pieces in hand. This now conforms to the USI format.
- Fixed a bug in calculate_checkers_and_pinned (Rook was treated as Bishop) which led to hallucinated checks.
- Added null-move implementation.
- Added support and instructions in haitaka/build.rs for building with `qugiy` feature flag.

## v0.2.0

### Changed (**breaking**)
- Project layout now is a workspace with two packages `haitaka_types` and `haitaka`. This was necessary in order to write the 'haitaka` build script that creates sliding moves tables at build time. The build script relies on `haitaka_types`. This layour now mirrors the `cozy-chess` layout.

### Added
- Support for slider move creation by Magic Bitboards. This is now the default implementation.

### Fixed
- Fixed several bugs in move creation. See git log for details.

## v0.1.0

Initial release to crates.io.
