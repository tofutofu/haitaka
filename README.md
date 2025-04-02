`sparrow`

## Rust Shogi move generation library

`sparrow` is a [Shogi](https://en.wikipedia.org/wiki/Shogi) move generation library written in Rust that aims to support fast move generation.
It is inspired by the beautifully designed [`cozy-chess`](https://github.com/analog-hors/cozy-chess) library written by [`analog-hors`](https://github.com/analog-hors). 
The layout of the modules is largely the same as the layout of `cozy-chess`. But since there are significant
differences between Shogi and International Chess, I adapted many parts of the implementation. 

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
- `BitBoard` uses `u128` instead of `u64` as backing to handle the 9x9 Shogi board
- Move generation handles both board moves and drops
- Move generation of sliders does not use Magic Bitboards, but is based on
  the beautiful [Qugiy algorithm]() as also used in [YaneuraOu](https://github.com/yaneurao/YaneuraOu).

## Crate features
- `std`: Enable features that require `std`. Currently only used for the `Error` trait.

## References
- [cozy-chess](https://github.com/analog-hors/cozy-chess)
- [Qugiy Appeal](https://www.apply.computer-shogi.org/wcsc31/appeal/Qugiy/appeal.pdf)
- [Qugiy in YaneuraOu](https://yaneuraou.yaneu.com/2021/12/03/qugiys-jumpy-effect-code-complete-guide/)
- [USI - Universal Shogi Interface](http://hgm.nubati.net/usi.html)
- [YaneuraOu](https://github.com/yaneurao/YaneuraOu)
