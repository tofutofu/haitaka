// Pseudo-attacks - attacks vectors on the empty board.

use crate::*;

/// This module defines the pseudo-attacks for non-sliding pieces.
///
/// Pseudo-attacks for [`king`] for `color` on `square`.
///
/// # Examples
///
/// ```
/// # use sparrow::*;
/// let mut square = Square::E5;
/// let mut black_attacks = king_attacks(Color::Black, square);
/// let mut white_attacks = king_attacks(Color::White, square);
///
/// assert_eq!(black_attacks, white_attacks);
/// assert_eq!(black_attacks, bitboard!{
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . X X X . . .
///     . . . X . X . . .
///     . . . X X X . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
/// });
///
/// black_attacks = gold_attacks(Color::Black, square);
/// white_attacks = gold_attacks(Color::White, square);
///
/// assert_ne!(black_attacks, white_attacks);
/// assert_eq!(black_attacks, bitboard! {
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . X X X . . .
///     . . . X . X . . .
///     . . . . X . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
/// });
/// assert_eq!(white_attacks, bitboard! {
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . X . . . .
///     . . . X . X . . .
///     . . . X X X . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
/// });
///
/// square = Square::I1;
/// black_attacks = king_attacks(Color::Black, square);
/// white_attacks = king_attacks(Color::White, square);
///
/// assert_eq!(black_attacks, white_attacks);
/// assert_eq!(black_attacks, bitboard!{
///     . X . . . . . . .
///     X X . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
/// });
///
/// square = Square::G2;
/// black_attacks = pawn_attacks(Color::Black, square);
/// assert_eq!(black_attacks.len(), 1);
/// assert!(black_attacks.has(Square::F2));
///
/// white_attacks = pawn_attacks(Color::White, square);
/// assert_eq!(white_attacks.len(), 1);
/// assert!(white_attacks.has(Square::H2));
///
/// square = Square::A1;
/// black_attacks = pawn_attacks(Color::Black, square);
/// assert!(black_attacks.is_empty());
///
/// square = Square::I1;
/// white_attacks = pawn_attacks(Color::White, square);
/// assert!(white_attacks.is_empty());
/// ```
// Macro to set up the attack vectors for non-sliding pieces.
macro_rules! define_pseudo_attack {
    ($name:ident, $src:expr, $black_pattern:expr, $white_pattern:expr) => {
        #[doc = concat!("Pseudo-attacks for [`", stringify!($name), "`] for `color` on `square`.")]
        pub const fn $name(color: Color, square: Square) -> BitBoard {
            const TABLE: [[BitBoard; Square::NUM]; Color::NUM] = {
                let mut table = [[BitBoard::EMPTY; Square::NUM]; Color::NUM];
                let mut sq: usize = 0;
                while sq < Square::NUM {
                    table[Color::White as usize][sq] =
                        $white_pattern.shift($src, Square::index_const(sq));
                    table[Color::Black as usize][sq] =
                        $black_pattern.shift($src, Square::index_const(sq));
                    sq += 1;
                }
                table
            };

            TABLE[color as usize][square as usize]
        }
    };
}

// Define the specific pseudo-legal attack functions using the macro
define_pseudo_attack!(
    king_attacks,
    Square::B2,
    bitboard! {
        . . . . . . X X X
        . . . . . . X * X
        . . . . . . X X X
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
    },
    bitboard! {
        . . . . . . X X X
        . . . . . . X * X
        . . . . . . X X X
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
    }
);

define_pseudo_attack!(
    gold_attacks,
    Square::B2,
    bitboard! {
        . . . . . . X X X
        . . . . . . X * X
        . . . . . . . X .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
    },
    bitboard! {
        . . . . . . . X .
        . . . . . . X * X
        . . . . . . X X X
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
    }
);

define_pseudo_attack!(
    silver_attacks,
    Square::B2,
    bitboard! {
        . . . . . . X X X
        . . . . . . . * .
        . . . . . . X . X
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
    },
    bitboard! {
        . . . . . . X . X
        . . . . . . . * .
        . . . . . . X X X
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
    }
);

define_pseudo_attack!(
    knight_attacks,
    Square::C2,
    bitboard! {
        . . . . . . X . X
        . . . . . . . . .
        . . . . . . . * .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
    },
    bitboard! {
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . * .
        . . . . . . . . .
        . . . . . . X . X
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
    }
);

define_pseudo_attack!(
    pawn_attacks,
    Square::B1,
    bitboard! {
        . . . . . . . . X
        . . . . . . . . *
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
    },
    bitboard! {
        . . . . . . . . .
        . . . . . . . . *
        . . . . . . . . X
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
    }
);
