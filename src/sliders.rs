// Sliders pseudo-attacks

use crate::*;

/// Returns the Rook blocker mask for the given square.
///
/// The Rook blocker mask is the bitboard in which all bits corresponding
/// to Rook rays are set, excluding bits for the edges and excluding the square.
pub const fn get_rook_relevant_blockers(square: Square) -> BitBoard {
    let rank_moves = square.rank().bitboard().0;
    let file_moves = square.file().bitboard().0;
    BitBoard((rank_moves ^ file_moves) & BitBoard::INNER.0)
}

/// Get Lance blocker mask.
pub const fn get_lance_relevant_blockers(square: Square, color: Color) -> BitBoard {
    let mut ray = BitBoard::EMPTY.0;
    let mut sq = square;
    let dy = match color {
        Color::White => 1,
        Color::Black => -1,
    };
    loop {
        // this could be optimized, but it's not on the critical hot path
        match sq.try_offset(0, dy) {
            Some(next_sq) => {
                ray |= next_sq.bitboard().0;
                sq = next_sq;
            }
            None => break,
        }
    }
    BitBoard(ray & BitBoard::INNER.0)
}

/// Get Bishop blocker mask.
pub const fn get_bishop_relevant_blockers(square: Square) -> BitBoard {
    let mut rays = BitBoard::EMPTY.0;
    let mut i = 0;
    while i < Square::NUM {
        let target = Square::index_const(i);
        let rd = (square.rank() as i8 - target.rank() as i8).abs();
        let fd = (square.file() as i8 - target.file() as i8).abs();
        if rd == fd && rd != 0 {
            rays |= 1 << i;
        }
        i += 1;
    }
    BitBoard(rays & BitBoard::INNER.0)
}

/// Returns a BitBoard with the slider moves, given an array of deltas.
const fn get_slider_moves(
    square: Square,
    mut blockers: BitBoard,
    deltas: &[(i8, i8); 4],
) -> BitBoard {
    blockers.0 &= !square.bitboard().0;
    let mut moves = BitBoard::EMPTY;
    let mut i = 0;
    while i < deltas.len() {
        let (dx, dy) = deltas[i];
        if dx == dy {
            break;
        }
        let mut square = square;
        while !blockers.has(square) {
            if let Some(sq) = square.try_offset(dx, dy) {
                square = sq;
                moves.0 |= square.bitboard().0;
            } else {
                break;
            }
        }
        i += 1;
    }
    moves
}

pub const fn get_rook_moves_slow(square: Square, blockers: BitBoard) -> BitBoard {
    get_slider_moves(square, blockers, &[(1, 0), (0, -1), (-1, 0), (0, 1)])
}

pub const fn get_bishop_moves_slow(square: Square, blockers: BitBoard) -> BitBoard {
    get_slider_moves(square, blockers, &[(1, 1), (1, -1), (-1, -1), (-1, 1)])
}

pub const fn get_lance_moves_slow(square: Square, blockers: BitBoard, color: Color) -> BitBoard {
    let dy = match color {
        Color::White => 1,
        Color::Black => -1,
    };
    get_slider_moves(square, blockers, &[(0, dy), (0, 0), (0, 0), (0, 0)])
}

// TODO: Should the TABLE arrays be lifted out of the function defs to avoid code bloat?

/// Rook pseudo-attacks from square.
///
/// # Examples
/// ```
/// use sparrow::*;
/// assert_eq!(rook_pseudo_attacks(Square::E5), bitboard! {
///     . . . . X . . . .
///     . . . . X . . . .
///     . . . . X . . . .
///     . . . . X . . . .
///     X X X X . X X X X
///     . . . . X . . . .
///     . . . . X . . . .
///     . . . . X . . . .
///     . . . . X . . . .
/// });
/// assert_eq!(rook_pseudo_attacks(Square::A9), bitboard! {
///     . X X X X X X X X
///     X . . . . . . . .
///     X . . . . . . . .
///     X . . . . . . . .
///     X . . . . . . . .
///     X . . . . . . . .
///     X . . . . . . . .
///     X . . . . . . . .
///     X . . . . . . . .
/// });
/// ```
#[inline]
pub const fn rook_pseudo_attacks(square: Square) -> BitBoard {
    const TABLE: [BitBoard; Square::NUM] = {
        let mut table = [BitBoard::EMPTY; Square::NUM];
        let mut index: usize = 0;
        while index < Square::NUM {
            let sq = Square::index_const(index);
            let rank_moves = sq.rank().bitboard().0;
            let file_moves = sq.file().bitboard().0;
            table[index] = BitBoard(rank_moves ^ file_moves);
            index += 1;
        }

        table
    };

    TABLE[square as usize]
}

/// Bishop pseudo-attacks.
///
/// # Examples
/// ```
/// use sparrow::*;
/// assert_eq!(bishop_pseudo_attacks(Square::E5), bitboard! {
///     X . . . . . . . X
///     . X . . . . . X .
///     . . X . . . X . .
///     . . . X . X . . .
///     . . . . * . . . .
///     . . . X . X . . .
///     . . X . . . X . .
///     . X . . . . . X .
///     X . . . . . . . X
/// });
/// assert_eq!(bishop_pseudo_attacks(Square::C3), bitboard! {
///     . . . . X . . . X
///     . . . . . X . X .
///     . . . . . . * . .
///     . . . . . X . X .
///     . . . . X . . . X
///     . . . X . . . . .
///     . . X . . . . . .
///     . X . . . . . . .
///     X . . . . . . . .
/// });
/// ```
#[inline]
pub const fn bishop_pseudo_attacks(square: Square) -> BitBoard {
    const TABLE: [BitBoard; Square::NUM] = {
        let mut table = [BitBoard::EMPTY; Square::NUM];
        let mut index: usize = 0;
        while index < Square::NUM {
            let sq = Square::index_const(index);
            table[index] = BitBoard(sq.up_diagonal().0 ^ sq.down_diagonal().0);
            index += 1;
        }

        table
    };

    TABLE[square as usize]
}

/// Lance pseudo-attacks.
///
/// # Examples
///
/// ```
/// use sparrow::*;
/// assert_eq!(lance_pseudo_attacks(Color::Black, Square::A1), BitBoard::EMPTY);
/// assert_eq!(lance_pseudo_attacks(Color::White, Square::I1), BitBoard::EMPTY);
/// assert_eq!(lance_pseudo_attacks(Color::Black, Square::A9), BitBoard::EMPTY);
/// assert_eq!(lance_pseudo_attacks(Color::White, Square::I9), BitBoard::EMPTY);
///
/// assert_eq!(lance_pseudo_attacks(Color::White, Square::A1), bitboard! {
///     . . . . . . . . *
///     . . . . . . . . X
///     . . . . . . . . X
///     . . . . . . . . X
///     . . . . . . . . X
///     . . . . . . . . X
///     . . . . . . . . X
///     . . . . . . . . X
///     . . . . . . . . X
/// });
/// assert_eq!(lance_pseudo_attacks(Color::White, Square::C9), bitboard! {
///     . . . . . . . . .
///     . . . . . . . . .
///     * . . . . . . . .
///     X . . . . . . . .
///     X . . . . . . . .
///     X . . . . . . . .
///     X . . . . . . . .
///     X . . . . . . . .
///     X . . . . . . . .
/// });
/// assert_eq!(lance_pseudo_attacks(Color::Black, Square::C9), bitboard! {
///     X . . . . . . . .
///     X . . . . . . . .
///     * . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
/// });
/// assert_eq!(lance_pseudo_attacks(Color::White, Square::H1), bitboard! {
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . *
///     . . . . . . . . X
/// });
/// assert_eq!(lance_pseudo_attacks(Color::White, Square::H5), bitboard! {
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . * . . . .
///     . . . . X . . . .
/// });
/// ```
#[inline]
pub const fn lance_pseudo_attacks(color: Color, square: Square) -> BitBoard {
    const TABLE: [[BitBoard; Square::NUM]; Color::NUM] = {
        let mut table = [[BitBoard::EMPTY; Square::NUM]; Color::NUM];

        let white = Color::White as usize;
        let black = Color::Black as usize;

        let mut north_attacks: u128 = 0x0;
        let mut south_attacks: u128 = 0x1FE;
        let mut mask: u128 = 0xFF;
        let mut index: usize = 0;

        while index < Square::NUM {
            if index % 9 == 0 {
                mask = 0x1FF << index;
            }

            table[black][index] = BitBoard(north_attacks & mask);
            table[white][index] = BitBoard(south_attacks & mask);

            north_attacks = (north_attacks << 1) | 0x1;
            south_attacks <<= 1;

            index += 1;
        }

        table
    };

    TABLE[color as usize][square as usize]
}

/// Return a BitBoard with pseudo-legal lance moves.
///
/// This returns a BitBoard with all the squares attacked by the lance,
/// up to and including the first blocker piece (if any).
///
/// The implementation uses the Qugiy algorithm.
///
/// # Example
/// ```
/// use sparrow::*;
/// let occ = bitboard! {
///      . . . . . X X X X
///      . . . . . . . X .
///      . . . . . X . X X
///      . . . . . . . . .
///      . . . . . . . . .
///      . . . . . . X . .
///      . . . . . . . . .
///      . . . . . X X X .
///      . . . . . X . X X
/// };
/// let mov1 = bitboard! {
///      . . . . . . * . .
///      . . . . . . X . .
///      . . . . . . X . .
///      . . . . . . X . .
///      . . . . . . X . .
///      . . . . . . X . .
///      . . . . . . . . .
///      . . . . . . . . .
///      . . . . . . . . .
/// };
/// assert_eq!(get_lance_moves(Color::White, Square::A3, occ), mov1);
/// let mov2 = bitboard! {
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . X
///     . . . . . . . . X
///     . . . . . . . . X
///     . . . . . . . . X
///     . . . . . . . . X
///     . . . . . . . . X
///     . . . . . . . . *
/// };
/// assert_eq!(get_lance_moves(Color::Black, Square::I1, occ), mov2);
/// ```
#[inline(always)]
pub const fn get_lance_moves(color: Color, square: Square, occ: BitBoard) -> BitBoard {
    //
    // Using the Qugiy algorithm -- as used in YaneuraOu
    //
    // Cost: 1 table lookup + 4 bit-operations
    // Extra cost for Black: + 3 bit reversals
    //
    let mut attacks = lance_pseudo_attacks(color, square).0;
    let mut occ = occ.0;

    if (attacks & occ) == 0 {
        // nothing is blocking the attacks (if there are any)
        return BitBoard(attacks);
    }

    match color {
        Color::White => BitBoard((((attacks & occ) - 1) ^ occ) & attacks),
        Color::Black => {
            attacks = attacks.reverse_bits();
            occ = occ.reverse_bits();
            BitBoard(((((attacks & occ) - 1) ^ occ) & attacks).reverse_bits())
        }
    }
}

/// Return a BitBoard of Rook moves on its file, up to the first blocking pieces (if any).
///
/// # Examples
/// ```
/// use sparrow::*;
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
/// let mov_e5 = bitboard! {
///     . . . . . . . . .
///     . . . . X . . . .
///     . . . . X . . . .
///     . . . . X . . . .
///     . . . . * . . . .
///     . . . . X . . . .
///     . . . . X . . . .
///     . . . . X . . . .
///     . . . . . . . . .
/// };
/// ```
#[inline(always)]
pub const fn get_rook_file_moves(square: Square, occ: BitBoard) -> BitBoard {
    let north = get_lance_moves(Color::Black, square, occ).0;
    let south = get_lance_moves(Color::White, square, occ).0;
    BitBoard(north | south)
}

// Rook attack masks - west- and east-bound rays along the board ranks.
// This array serves the same function as the QUGIY_ROOK_MASK table in YaneuraOu.

const ROOK_RANK_MASKS: [(u128, u128); Square::NUM] = {
    let mut masks = [(0u128, 0u128); Square::NUM];
    let mut index = 0;
    while index < Square::NUM {
        let square = Square::index_const(index);
        let file = square.file();
        let rank_bb = square.rank().bitboard().0;

        // West mask: All bits to the west (higher bits) of the square
        let west_mask = rank_bb & file.west().0;

        // East mask: All bits to the east (lower bits) of the square
        let east_mask = rank_bb & file.east().0;

        masks[index] = (west_mask, east_mask.reverse_bits());
        index += 1;
    }
    masks
};

/// Return a BitBoard of Rook moves on its rank, up to the first blocking pieces (if any).
///
/// # Examples
/// ```
/// use sparrow::*;
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
/// let mov = bitboard! {
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . X X X * X X X .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
/// };
/// assert_eq!(get_rook_rank_moves(Square::E5, occ), mov);
/// ```
#[inline(always)]
pub const fn get_rook_rank_moves(square: Square, occ: BitBoard) -> BitBoard {
    let (mut west_attacks, mut east_attacks) = ROOK_RANK_MASKS[square as usize];

    let mut index = (west_attacks & occ.0).trailing_zeros();
    if index < 127 {
        west_attacks = ((1 << (index + 1)) - 1) & west_attacks;
    }

    index = (east_attacks & occ.0.reverse_bits()).trailing_zeros();
    if index < 127 {
        east_attacks = ((1 << (index + 1)) - 1) & east_attacks;
    }

    BitBoard::new(west_attacks | east_attacks.reverse_bits())
}

/// Get rook moves.
///
/// # Examples
/// ```
/// use sparrow::*;
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
/// assert_eq!(get_rook_moves(Square::E5, occ), e5_attacks);
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
/// assert_eq!(get_rook_moves(Square::H5, occ), h5_attacks);
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
/// assert_eq!(get_rook_moves(Square::C7, occ), c7_attacks);
/// ```
#[inline(always)]
pub const fn get_rook_moves(square: Square, occ: BitBoard) -> BitBoard {
    let bb1 = get_rook_rank_moves(square, occ);
    let bb2 = get_rook_file_moves(square, occ);
    bb1.bitor(bb2)
}

// Bishop attack rays
//
//  NW    NE
//     sq
//  SW    SE
//
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

// Layout
//
//  NW    NE
//     +
//  SW    SE
//

/// Get bishop moves.
///
/// This applies the Qugiy algorithm to calculate the Bishop pseudo-legal moves, given a position.
/// ```text
/// # occ = occupancy bits
/// # attacks = ray attack bits (with bit indices greater than square index)
/// BitBoard((((attacks & occ) - 1) ^ occ) & attacks)
/// ```
/// This algorithm can only apply to attack rays with bit indices greater than the square index.
/// So, the east-wards (right-wards) rays have been reversed.
///
/// # Examples
/// ```
/// use sparrow::*;
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
/// assert_eq!(get_bishop_moves(Square::E5, occ), e5_attacks);
/// ```
#[inline(always)]
pub const fn get_bishop_moves(square: Square, occ: BitBoard) -> BitBoard {
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
