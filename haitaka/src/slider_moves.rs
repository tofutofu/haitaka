//! Slider
//!
//! Move generation of slider moves uses the Qugiy algoritm (introduced for WCSC31 by Mori Taikei)
//! if the `qugiy` feature flag is set. If this flag is not set we use Magic Bitboard tables to
//! generate Bishop and Rook moves.
//! 
//! The Qugiy algorithm uses a trick very similar to the [carry rippler](https://www.chessprogramming.org/Traversing_Subsets_of_a_Set#All_Subsets_of_any_Set) 
//! to determine which squares up to the first blocker are unoccupied on Rook or Bishop rays. It amounts to the
//! following operations
//! ```text
//!     # occ = occupancy bits
//!     # attacks = ray attack bits (with bit indices greater than square index)
//!     BitBoard((((attacks & occ) - 1) ^ occ) & attacks)
//! ```
//! This algorithm can only be applied to attack rays in which all square have bit indices greater than
//! the index of the square on which the slider stands. For rays in the opposite direction, we first need 
//! to reverse the bitsets. The current implementation doesn't use intrinsics, so there may still be some
//! room to make it faster by using intrinsics with implicit data parallellism. As it stands, our qugiy
//! implementation is about 3x as slow as the default implementation using Magic BitBoards. On the other
//! hand, it doesn't need to allocate a huge amount of extra memory for the moves tables (see 
//! `SLIDING_MOVES_TABLE_SIZE` in `haitaka_types/src/sliders/magic.rs`).
//!

use crate::*;

#[cfg(not(feature = "qugiy"))]
include!(concat!(env!("OUT_DIR"), "/sliding_moves_table.rs"));

/// Get rook moves.
///
/// # Examples
/// ```
/// use haitaka::*;
/// let occ = bitboard! {
///     . . . . . . . . .
///     . . . . X . . X .
///     . . X . . . . . .
///     . . . . . . . . .
///     X X . . X . . X .
///     . . . . . . . . .
///     . . . . . . X . .
///     . X . . X . . . .
///     . . . . . . . . .
/// };
/// let e5_attacks = bitboard! {
///     . . . . . . . . .
///     . . . . X . . . .
///     . . . . X . . . .
///     . . . . X . . . .
///     . X X X . X X X .
///     . . . . X . . . .
///     . . . . X . . . .
///     . . . . X . . . .
///     . . . . . . . . .
/// };
/// assert_eq!(get_rook_moves(Color::White, Square::E5, occ), e5_attacks);
///
/// let h5_attacks = bitboard! {
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . X . . . .
///     . . . . X . . . .
///     . . . . X . . . .
///     . X X X * X X X X
///     . . . . X . . . .
/// };
/// assert_eq!(get_rook_moves(Color::White, Square::H5, occ), h5_attacks);
///
/// let c7_attacks = bitboard! {
///     . . X . . . . . .
///     . . X . . . . . .
///     X X * X X X X X X
///     . . X . . . . . .
///     . . X . . . . . .
///     . . X . . . . . .
///     . . X . . . . . .
///     . . X . . . . . .
///     . . X . . . . . .
/// };
/// assert_eq!(get_rook_moves(Color::White, Square::C7, occ), c7_attacks);
/// ```
#[inline(always)]
pub const fn get_rook_moves(_color: Color, square: Square, occ: BitBoard) -> BitBoard {
    // The _color argument is not used, but added for consistency in function signatures.
    #[cfg(feature = "qugiy")]
    {
        let bb1 = get_rook_rank_moves(square, occ);
        let bb2 = get_rook_file_moves(square, occ);
        bb1.bitor(bb2)
    }
    #[cfg(not(feature = "qugiy"))]
    {
        BitBoard(ROOK_MOVES[get_rook_moves_index(square, occ)])
    }
}

// Bishop attack rays
//
//  NW    NE
//     sq
//  SW    SE
//
#[cfg(feature = "qugiy")]
const BISHOP_RAY_MASKS: [(u128, u128, u128, u128); Square::NUM] = {
    let mut masks = [(0u128, 0u128, 0u128, 0u128); Square::NUM];
    let mut index = 0;
    while index < Square::NUM {
        let square = Square::index_const(index);
        let file = square.file();
        let rank = square.rank();

        let up = square.up_diagonal(); // forward slashing '/'
        let down = square.down_diagonal(); // back slashing '\'      

        let nw = down.bitand(rank.north().bitand(file.west())).0;
        let ne = up.bitand(rank.north().bitand(file.east())).0;
        let sw = up.bitand(rank.south().bitand(file.west())).0;
        let se = down.bitand(rank.south().bitand(file.east())).0;

        masks[index] = (nw, ne.reverse_bits(), sw, se.reverse_bits());

        index += 1;
    }
    masks
};

/// Get bishop moves.
///
/// # Examples
/// ```
/// use haitaka::*;
/// let occ = bitboard! {
///     . . . . . . . . .
///     . . . . X . . X .
///     . . X . . . . . .
///     . . . . . . . . .
///     X X . . . . . X .
///     . . . . . . . . .
///     . . . . . . X . .
///     . X . . X . . . .
///     . . . . . . . . .
/// };
/// let e5_attacks = bitboard! {
///     . . . . . . . . .
///     . . . . . . . X .
///     . . X . . . X . .
///     . . . X . X . . .
///     . . . . * . . . .
///     . . . X . X . . .
///     . . X . . . X . .
///     . X . . . . . . .
///     . . . . . . . . .
/// };
/// assert_eq!(get_bishop_moves(Color::White, Square::E5, occ), e5_attacks);
/// ```
#[inline(always)]
pub const fn get_bishop_moves(_color: Color, square: Square, occ: BitBoard) -> BitBoard {
    #[cfg(feature = "qugiy")]
    {
        // The _color argument is not used, but added for consistency in function signatures.
        let (mut nw, mut ne_rev, mut sw, mut se_rev) = BISHOP_RAY_MASKS[square as usize];

        let occ = occ.0;
        let occ_rev = occ.reverse_bits();

        // Rust panics on arithmetic under/overflows ...
        // TODO: Should I switch to an i128 base type to be able to skip these tests? :/
        if (nw & occ) != 0 {
            nw = (((nw & occ) - 1) ^ occ) & nw;
        }

        if (sw & occ) != 0 {
            sw = (((sw & occ) - 1) ^ occ) & sw;
        }

        if (ne_rev & occ_rev) != 0 {
            ne_rev = (((ne_rev & occ_rev) - 1) ^ occ_rev) & ne_rev;
        }

        if (se_rev & occ_rev) != 0 {
            se_rev = (((se_rev & occ_rev) - 1) ^ occ_rev) & se_rev;
        }

        BitBoard(nw | sw | ne_rev.reverse_bits() | se_rev.reverse_bits())
    }
    #[cfg(not(feature = "qugiy"))]
    {
        BitBoard(BISHOP_MOVES[get_bishop_moves_index(square, occ)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rook() {
        let occ = BitBoard::EMPTY;
        let expected = rook_pseudo_attacks(Square::E5);
        let actual = get_rook_moves_slow(Square::E5, occ);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_relevant_blockers() {
        let bb = get_rook_relevant_blockers(Square::E5);
        assert!(!bb.has(Square::E5));
        let bb = get_bishop_relevant_blockers(Square::E5);
        assert!(!bb.has(Square::E5));
        let bb = get_lance_relevant_blockers(Square::E5, Color::Black);
        assert!(!bb.has(Square::E5));
        let bb = get_lance_relevant_blockers(Square::E5, Color::White);
        assert!(!bb.has(Square::E5));
    }

    #[test]
    fn test_bishop_fast_and_slow_e5() {
        let occ = BitBoard::EMPTY;
        let expected = bishop_pseudo_attacks(Square::E5);

        let actual1 = get_bishop_moves_slow(Square::E5, occ);
        assert_eq!(actual1, expected);

        let actual2 = get_bishop_moves(Color::White, Square::E5, occ);
        assert_eq!(actual2, expected);
    }

    #[test]
    fn test_bishop_fast_and_slow_g3() {
        let occ = BitBoard::EMPTY;
        let expected = bishop_pseudo_attacks(Square::G3);
        let actual1 = get_bishop_moves_slow(Square::G3, occ);
        assert_eq!(actual1, expected);

        let actual2 = get_bishop_moves(Color::White, Square::G3, occ);
        assert_eq!(actual2, expected);
    }

    #[test]
    fn test_bishop_fast_and_slow_a1() {
        let occ = BitBoard::EMPTY;
        let expected = bishop_pseudo_attacks(Square::A1);
        let actual1 = get_bishop_moves_slow(Square::A1, occ);
        assert_eq!(actual1, expected);

        let actual2 = get_bishop_moves(Color::White, Square::A1, occ);
        assert_eq!(actual2, expected);
    }

    #[test]
    fn test_bishop_moves() {
        let occ = bitboard! {
            . . . . . . . . .
            . . . . X . . X .
            . . X . . . . . .
            . . . . . . . . .
            X X . . X . . X .
            . . . . . . . . .
            . . . . . . X . .
            . X . . X . . . .
            . . . . . . . . .
        };
        let e5_attacks = bitboard! {
            . . . . . . . . .
            . . . . . . . X .
            . . X . . . X . .
            . . . X . X . . .
            . . . . * . . . .
            . . . X . X . . .
            . . X . . . X . .
            . X . . . . . . .
            . . . . . . . . .
        };
        let h8_attacks = bitboard! {
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . X . . . .
            . . . X . . . . .
            X . X . . . . . .
            . * . . . . . . .
            X . X . . . . . .
        };
        let g3_attacks = bitboard! {
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . X . . . X
            . . . . . X . X .
            . . . . . . * . .
            . . . . . X . X .
            . . . . X . . . X
        };

        let actual = get_bishop_moves(Color::White, Square::E5, occ);
        assert_eq!(actual, e5_attacks);
        let actual = get_bishop_moves(Color::Black, Square::E5, occ);
        assert_eq!(actual, e5_attacks);
        let actual = get_bishop_moves(Color::White, Square::H8, occ);
        assert_eq!(actual, h8_attacks);

        assert!(occ.has(Square::G3));
        assert!(Square::G3.up_diagonal().has(Square::G3));
        assert!(Square::G3.down_diagonal().has(Square::G3));
        let actual1 = get_bishop_moves_slow(Square::G3, occ);
        assert_eq!(actual1, g3_attacks);

        let actual2 = get_bishop_moves(Color::White, Square::G3, occ);
        assert_eq!(actual2, g3_attacks);
    }
}
