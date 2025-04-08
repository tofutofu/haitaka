//! Pseudo-attacks for non-sliding pieces
// The macro could actually also be used to set up the attack tables for sliders,
// but at the moment this is done differently (see: sliders.rs).
use crate::*;

// Macro to set up the attack vectors for non-sliding pieces.
macro_rules! define_pseudo_attack {
    ($name:ident, $src:expr, $black_pattern:expr, $white_pattern:expr) => {
        #[doc = concat!("Pseudo-attacks for [`", stringify!($name), "`] for `color` on `square`.")]
        #[inline(always)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_king_attacks() {
        // middle of board
        let square = Square::E5;
        let black_attacks = king_attacks(Color::Black, square);
        let white_attacks = king_attacks(Color::White, square);

        assert_eq!(black_attacks, white_attacks);
        assert_eq!(
            black_attacks,
            bitboard! {
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . X X X . . .
                . . . X * X . . .
                . . . X X X . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
            }
        );

        // corner
        let square = Square::A9;
        let black_attacks = king_attacks(Color::Black, square);
        let white_attacks = king_attacks(Color::White, square);

        assert_eq!(black_attacks, white_attacks);
        assert_eq!(
            black_attacks,
            bitboard! {
                * X . . . . . . .
                X X . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
            }
        );
    }

    #[test]
    fn test_gold_attacks() {
        let square = Square::E5;
        let black_attacks = gold_attacks(Color::Black, square);
        let white_attacks = gold_attacks(Color::White, square);

        assert_ne!(black_attacks, white_attacks);
        assert_eq!(
            black_attacks,
            bitboard! {
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . X X X . . .
                . . . X * X . . .
                . . . . X . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
            }
        );
        assert_eq!(
            white_attacks,
            bitboard! {
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . X . . . .
                . . . X * X . . .
                . . . X X X . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
            }
        );

        let square = Square::B1;
        let black_attacks = gold_attacks(Color::Black, square);
        let white_attacks = gold_attacks(Color::White, square);

        assert_ne!(black_attacks, white_attacks);
        assert_eq!(
            black_attacks,
            bitboard! {
                . . . . . . . X X
                . . . . . . . X *
                . . . . . . . . X
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
            }
        );
        assert_eq!(
            white_attacks,
            bitboard! {
                . . . . . . . . X
                . . . . . . . X *
                . . . . . . . X X
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
            }
        );
    }

    #[test]
    fn test_silver_attacks() {
        let square = Square::E5;
        let black_attacks = silver_attacks(Color::Black, square);
        let white_attacks = silver_attacks(Color::White, square);

        assert_ne!(black_attacks, white_attacks);
        assert_eq!(
            black_attacks,
            bitboard! {
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . X X X . . .
                . . . . * . . . .
                . . . X . X . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
            }
        );
        assert_eq!(
            white_attacks,
            bitboard! {
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . X . X . . .
                . . . . * . . . .
                . . . X X X . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
            }
        );

        let square = Square::H9;
        let black_attacks = silver_attacks(Color::Black, square);
        let white_attacks = silver_attacks(Color::White, square);

        assert_ne!(black_attacks, white_attacks);
        assert_eq!(
            black_attacks,
            bitboard! {
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                X X . . . . . . .
                * . . . . . . . .
                . X . . . . . . .
            }
        );
        assert_eq!(
            white_attacks,
            bitboard! {
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . X . . . . . . .
                * . . . . . . . .
                X X . . . . . . .
            }
        );
    }

    #[test]
    fn test_knight_attacks() {
        let square = Square::E5;
        let black_attacks = knight_attacks(Color::Black, square);
        let white_attacks = knight_attacks(Color::White, square);

        assert_ne!(black_attacks, white_attacks);
        assert_eq!(
            black_attacks,
            bitboard! {
                . . . . . . . . .
                . . . . . . . . .
                . . . X . X . . .
                . . . . . . . . .
                . . . . * . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
            }
        );
        assert_eq!(
            white_attacks,
            bitboard! {
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . * . . . .
                . . . . . . . . .
                . . . X . X . . .
                . . . . . . . . .
                . . . . . . . . .
            }
        );

        let square = Square::C1;
        let black_attacks = knight_attacks(Color::Black, square);
        let white_attacks = knight_attacks(Color::White, square);

        assert_eq!(
            black_attacks,
            bitboard! {
                . . . . . . . X .
                . . . . . . . . .
                . . . . . . . . *
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
            }
        );
        assert_eq!(
            white_attacks,
            bitboard! {
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . *
                . . . . . . . . .
                . . . . . . . X .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
            }
        );
    }

    #[test]
    fn test_pawn_attacks() {
        let square = Square::E5;
        let black_attacks = pawn_attacks(Color::Black, square);
        let white_attacks = pawn_attacks(Color::White, square);

        assert_ne!(black_attacks, white_attacks);
        assert_eq!(
            black_attacks,
            bitboard! {
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . X . . . .
                . . . . * . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
            }
        );
        assert_eq!(
            white_attacks,
            bitboard! {
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . * . . . .
                . . . . X . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
            }
        );

        let square = Square::A1;
        let black_attacks = pawn_attacks(Color::Black, square);
        let white_attacks = pawn_attacks(Color::White, square);

        assert_ne!(black_attacks, white_attacks);
        assert_eq!(
            black_attacks,
            bitboard! {
                . . . . . . . . *
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
            }
        );
        assert_eq!(
            white_attacks,
            bitboard! {
                . . . . . . . . *
                . . . . . . . . X
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
                . . . . . . . . .
            }
        );
    }
}
