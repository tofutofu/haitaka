//! Slider 
//! 
//! Move generation of slider moves will use the Qugiy algoritm (introduced for WCSC31 by Mori Taikei)
//! if the `qugiy` feature flag is set. If this flag is not set we use magic bigboard hash tables
//! to generate Bishop and Rook moves.
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
#[cfg(feature = "qugiy")]
#[inline(always)]
pub const fn get_rook_moves(_color: Color, square: Square, occ: BitBoard) -> BitBoard {
    // The _color argument is not used, but added for consistency in function signatures.
    let bb1 = get_rook_rank_moves(square, occ);
    let bb2 = get_rook_file_moves(square, occ);
    bb1.bitor(bb2)
}

#[cfg(not(feature = "qugiy"))]
#[inline(always)]
pub fn get_rook_moves(_color: Color, square: Square, blockers: BitBoard) -> BitBoard {
    BitBoard(SLIDING_MOVES[get_rook_moves_index(square, blockers)])
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
/// This applies the Qugiy algorithm to calculate the Bishop pseudo-legal moves, given a position.
/// ```text
/// # occ = occupancy bits
/// # attacks = ray attack bits (with bit indices greater than square index)
/// BitBoard((((attacks & occ) - 1) ^ occ) & attacks)
/// ```
/// This algorithm can only apply to attack rays with bit indices greater than the square index.
/// So, the east-wards (right-wards) rays are reversed during the calculation.
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
///     . . . . . . . X .
///     . . X . . . X . .
///     . . . X . X . . .
///     . . . . * . . . .
///     . . . X . X . . .
///     . . X . . . X . .
///     . X . . . . . . .
///     . . . . . . . . .
/// };
/// let h8_attacks = bitboard! {
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . X . . . .
///     . . . X . . . . .
///     X . X . . . . . .
///     . * . . . . . . .
///     X . X . . . . . .
/// };
/// let g3_attacks = bitboard! {
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . X . . . X
///     . . . . . X . X .
///     . . . . . . * . .
///     . . . . . X . X .
///     . . . . X . . . X
/// };
/// assert_eq!(get_bishop_moves(Color::White, Square::E5, occ), e5_attacks);
/// ```
#[cfg(feature = "qugiy")]
#[inline(always)]
pub const fn get_bishop_moves(_color: Color, square: Square, occ: BitBoard) -> BitBoard {
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
#[inline(always)]
pub fn get_bishop_moves(_color: Color, square: Square, blockers: BitBoard) -> BitBoard {
    BitBoard(SLIDING_MOVES[get_bishop_moves_index(square, blockers)])
}


