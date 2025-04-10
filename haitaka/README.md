`haitaka` `ハイタカ`

## Rust Shogi move generation library

`haitaka` is a [Shogi](https://en.wikipedia.org/wiki/Shogi) move generation library written in Rust that aims to support fast move generation.
It is inspired by the beautifully designed [`cozy-chess`](https://github.com/analog-hors/cozy-chess) library written by [`analog-hors`](https://github.com/analog-hors). 
The layout of the modules and the overall design is largely the same as in `cozy-chess`. Many low-level functions were copied from `cozy-chess`, with only trivial modifications, but since there are significant differences between Shogi and International Chess, I also modified some of the higher-level functions and added extra functionality.

## Name

"Haitaka" or "taka" means "sparrowhawk" in Japanese. "Taka" is a haiku _kigo_ (season word) associated with　winter.

   鷹の眼​にこぼれて雁のたち騒ぐ<br>
   _Taka no me ni koborete kari no tachisawagu_

   _​Escaping the hawk's eye,<br>
   the wild geese<br>
   rise in tumult._<br>
   -— 加賀千代女 ([Kaga no Chiyojo](https://en.wikipedia.org/wiki/Fukuda_Chiyo-ni))

## Overview
- `no_std` compatible
- Strongly-typed API that makes heavy use of newtypes to avoid errors
- Efficient bitboard-based board representation
- Performant legal move generation
- Incrementally updated zobrist hash for quickly obtaining a hash of a board
- Supporting both Magic Bitboards and the [Qugiy](https://www.apply.computer-shogi.org/wcsc31/appeal/Qugiy/appeal.pdf) algorithm for slider move generation
- Support for parsing [SFEN] strings

## Main differences with `cozy-chess`
- `BitBoard` uses `u128` instead of `u64` as backing to handle the 9x9 Shogi board
- Move generation handles both board moves and drops
- Move generation of sliders also implements the [Qugiy algorithm](https://yaneuraou.yaneu.com/2021/12/03/qugiys-jumpy-effect-code-complete-guide/)
- File-major ordering of squares to make move generation faster

## Crate features
- `std`: Enable features that require `std`. Currently only used for the `Error` trait.

## Installation
Add `haitaka` to your `Cargo.toml`:
```toml
[dependencies]
haitaka = "0.2.0"   # or use the latest version on crates.io
```

## Usage

### Basic 
```rust
// Start position
let board = Board::startpos();
let mut move_list = Vec::new();
board.generate_moves(|moves| {
    // Unpack dense move set into move list
    move_list.extend(moves);
    false
});
assert_eq!(move_list.len(), 20);
```

### Perft
```bash
cargo run --release --example perft -- 5
```

## Testing

This code has been tested yet on an Apple M2, using the stable-aarch64-apple-darwin toolchain. In
GitHub workflows it has also been tested on Ubuntu.

The code should still be seen as experimental since it has not yet been stress tested or used in an actual Shogi engine. I'm also still working on experimental optimizations.

To run all tests use:
```bash
cargo test
```

## Acknowledgments
Portions of this library are derived from the [`cozy-chess`](https://github.com/analog-hors/cozy-chess) project by [`analog-hors`](https://github.com/analog-hors). The `cozy-chess` project is licensed under the MIT license, and its license text is included in this repository under `third_party/cozy-chess/LICENSE`.

## References
- [cozy-chess](https://github.com/analog-hors/cozy-chess)
- [WCSC31 Qugiy Appeal](https://www.apply.computer-shogi.org/wcsc31/appeal/Qugiy/appeal.pdf)
- [Qugiy in YaneuraOu](https://yaneuraou.yaneu.com/2021/12/03/qugiys-jumpy-effect-code-complete-guide/)
- [USI - Universal Shogi Interface](http://hgm.nubati.net/usi.html)
- [YaneuraOu](https://github.com/yaneurao/YaneuraOu)
