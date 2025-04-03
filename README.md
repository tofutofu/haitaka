`sparrow`

## Rust Shogi move generation library

`sparrow` is a [Shogi](https://en.wikipedia.org/wiki/Shogi) move generation library written in Rust that aims to support fast move generation.
It is inspired by the beautifully designed [`cozy-chess`](https://github.com/analog-hors/cozy-chess) library written by [`analog-hors`](https://github.com/analog-hors). 
The layout of the modules and the overall design is largely the same as in `cozy-chess`. Some low-level functions were also copied from `cozy-chess`, but since there are significant differences between Shogi and International Chess, I did modify or reimplement most of the higher-level functions.

## Overview
- `no_std` compatible
- Strongly-typed API that makes heavy use of newtypes to avoid errors
- Efficient bitboard-based board representation
- Performant legal move generation
- Incrementally updated zobrist hash for quickly obtaining a hash of a board
- Using the [Qugiy]() algorithm for lightweight and fast slider move generation
  (but not _yet_ supporting intrinsics)
- Support for [USI - Universal Shogi Interface](http://hgm.nubati.net/usi.html) protocol

## Main differences with `cozy-chess`
- No separate `types` crate
- `BitBoard` uses `u128` instead of `u64` as backing to handle the 9x9 Shogi board
- Move generation handles both board moves and drops
- Move generation of sliders does not use Magic Bitboards, but is based on
  the [Qugiy algorithm]() which is also used in [YaneuraOu](https://github.com/yaneurao/YaneuraOu).
- File-major ordering of squares to make move generation faster

## Crate features
- `std`: Enable features that require `std`. Currently only used for the `Error` trait.

## Acknowledgments
Portions of this library are derived from the [`cozy-chess`](https://github.com/analog-hors/cozy-chess) project by [`analog-hors`](https://github.com/analog-hors). The `cozy-chess` project is licensed under the MIT license, and its license text is included in this repository under `third_party/cozy-chess/LICENSE`.

## References
- [cozy-chess](https://github.com/analog-hors/cozy-chess)
- [Qugiy Appeal](https://www.apply.computer-shogi.org/wcsc31/appeal/Qugiy/appeal.pdf)
- [Qugiy in YaneuraOu](https://yaneuraou.yaneu.com/2021/12/03/qugiys-jumpy-effect-code-complete-guide/)
- [USI - Universal Shogi Interface](http://hgm.nubati.net/usi.html)
- [YaneuraOu](https://github.com/yaneurao/YaneuraOu)
