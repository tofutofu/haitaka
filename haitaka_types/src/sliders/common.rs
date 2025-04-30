//! Sliders pseudo-attack functions
use crate::*;

/// Returns the Rook blocker mask for the given square.
///
/// The Rook blocker mask is the bitboard in which all bits corresponding
/// to Rook rays are set, _excluding_ bits for the edges (unless the Rook is
/// on that edge) and excluding the Rook square.
pub const fn get_rook_relevant_blockers(square: Square) -> BitBoard {
    let rank_moves =
        square.rank().bitboard().0 & !(File::One.bitboard().0 | File::Nine.bitboard().0);
    let file_moves = square.file().bitboard().0 & !(Rank::A.bitboard().0 | Rank::I.bitboard().0);
    BitBoard::new((rank_moves ^ file_moves) & !square.bitboard().0)
}

/// Get Lance blocker mask.
pub const fn get_lance_relevant_blockers(square: Square, color: Color) -> BitBoard {
    let mut ray = BitBoard::EMPTY.0;
    let mut sq = square;
    let dy = match color {
        Color::White => 1,
        Color::Black => -1,
    };
    // this could be optimized, but it's not on the critical hot path
    while let Some(next_sq) = sq.try_offset(0, dy) {
        ray |= next_sq.bitboard().0;
        sq = next_sq;
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
const fn get_slider_moves(square: Square, blockers: BitBoard, deltas: &[(i8, i8); 4]) -> BitBoard {
    let blockers = blockers.rm(square);
    let mut moves = BitBoard::EMPTY;
    let mut i = 0;
    while i < deltas.len() {
        let (dx, dy) = deltas[i];
        if dx == 0 && dy == 0 {
            // sentinel for the Lance
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

/// Rook pseudo-attacks from square.
///
/// # Examples
/// ```
/// use haitaka_types::*;
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
/// use haitaka_types::*;
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
/// assert_eq!(bishop_pseudo_attacks(Square::G3), bitboard! {
///     X . . . . . . . .
///     . X . . . . . . .
///     . . X . . . . . .
///     . . . X . . . . .
///     . . . . X . . . X
///     . . . . . X . X .
///     . . . . . . * . .
///     . . . . . X . X .
///     . . . . X . . . X
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
/// use haitaka_types::*;
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
/// use haitaka_types::*;
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
    // Using this method is about 20% faster than trying to use the Rook magic move
    // table and masking out invalid ranks.
    //
    let mut attacks = lance_pseudo_attacks(color, square).0;
    let mut occ = occ.0;
    let aok = attacks & occ;

    if aok == 0 {
        // nothing is blocking the attacks
        return BitBoard(attacks);
    }

    match color {
        Color::White => BitBoard(((aok - 1) ^ occ) & attacks),
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
/// use haitaka_types::*;
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

// Rook ray attack masks - along ranks.
//
// Directions: West sq East
//
// This array serves the same function as the QUGIY_ROOK_MASK table in YaneuraOu.
//
const ROOK_RANK_MASKS: [(u128, u128); Square::NUM] = {
    let mut masks = [(0u128, 0u128); Square::NUM];
    let mut index = 0;
    while index < Square::NUM {
        let square = Square::index_const(index);
        let file = square.file();
        let rank = square.rank();
        let rnk = rank.bitboard().0;

        // West mask: All bits to the west (higher bits) of the square
        let west = rnk & file.west().0;

        // East mask: All bits to the east (lower bits) of the square
        let east = rnk & file.east().0;

        masks[index] = (west, east.reverse_bits());
        index += 1;
    }
    masks
};

/// Return a BitBoard of Rook moves on its rank, up to the first blocking pieces (if any).
///
/// # Examples
/// ```
/// use haitaka_types::*;
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
        west_attacks &= (1 << (index + 1)) - 1;
    }

    index = (east_attacks & occ.0.reverse_bits()).trailing_zeros();
    if index < 127 {
        east_attacks &= (1 << (index + 1)) - 1;
    }

    BitBoard::new(west_attacks | east_attacks.reverse_bits())
}

/// Get all squares between two squares, if reachable via a ray.
/// The `from` and `to` square are not included in the returns [`BitBoard`].
///
/// # Examples
/// ```
/// # use haitaka_types::*;
/// let rays = get_between_rays(Square::E2, Square::E7);
/// assert_eq!(rays, bitboard! {
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . X X X X . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
/// });
/// let no_rays = get_between_rays(Square::A1, Square::B3);
///  assert_eq!(no_rays, BitBoard::EMPTY);
/// ```
#[inline(always)]
pub const fn get_between_rays(from: Square, to: Square) -> BitBoard {
    const fn get_between_rays(from: Square, to: Square) -> BitBoard {
        let dx = to.file() as i8 - from.file() as i8; // -8 .. 8
        let dy = to.rank() as i8 - from.rank() as i8; // -8 .. 8
        let orthogonal = dx == 0 || dy == 0;
        let diagonal = dx.abs() == dy.abs();
        if !(orthogonal || diagonal) {
            return BitBoard::EMPTY;
        }
        let dx = dx.signum(); // -1, 0, 1
        let dy = dy.signum();
        let mut square = from.offset(dx, dy);
        let mut between = BitBoard::EMPTY;
        while square as u8 != to as u8 {
            between.0 |= square.bitboard().0;
            square = square.offset(dx, dy);
        }
        between
    }
    #[allow(clippy::large_const_arrays)]
    const TABLE: [[BitBoard; Square::NUM]; Square::NUM] = {
        let mut table = [[BitBoard::EMPTY; Square::NUM]; Square::NUM];
        let mut i = 0;
        while i < table.len() {
            let mut j = 0;
            while j < table[i].len() {
                table[i][j] = get_between_rays(Square::index_const(i), Square::index_const(j));
                j += 1;
            }
            i += 1;
        }
        table
    };
    TABLE[from as usize][to as usize]
}

/// Get a ray on the board that passes through both squares, if it exists.
///
/// These rays include the `from` and `to` square.
///
/// # Examples
/// ```
/// # use haitaka_types::*;
/// let rays = line_ray(Square::B1, Square::I8);
/// assert_eq!(rays, bitboard! {
///     . . . . . . . . .
///     . . . . . . . . X
///     . . . . . . . X .
///     . . . . . . X . .
///     . . . . . X . . .
///     . . . . X . . . .
///     . . . X . . . . .
///     . . X . . . . . .
///     . X . . . . . . .
/// });
/// assert_eq!(line_ray(Square::F5, Square::E4), line_ray(Square::B1, Square::I8));
/// assert_eq!(line_ray(Square::A3, Square::I3), line_ray(Square::C3, Square::F3));
/// ```
#[inline(always)]
pub const fn line_ray(from: Square, to: Square) -> BitBoard {
    const fn get_line_rays(from: Square, to: Square) -> BitBoard {
        let rays = bishop_pseudo_attacks(from);
        if rays.has(to) {
            return BitBoard(
                (rays.0 | from.bitboard().0) & (bishop_pseudo_attacks(to).0 | to.bitboard().0),
            );
        }
        let rays = rook_pseudo_attacks(from);
        if rays.has(to) {
            return BitBoard(
                (rays.0 | from.bitboard().0) & (rook_pseudo_attacks(to).0 | to.bitboard().0),
            );
        }
        BitBoard::EMPTY
    }
    #[allow(clippy::large_const_arrays)]
    const TABLE: [[BitBoard; Square::NUM]; Square::NUM] = {
        let mut table = [[BitBoard::EMPTY; Square::NUM]; Square::NUM];
        let mut i = 0;
        while i < table.len() {
            let mut j = 0;
            while j < table[i].len() {
                table[i][j] = get_line_rays(Square::index_const(i), Square::index_const(j));
                j += 1;
            }
            i += 1;
        }
        table
    };
    TABLE[from as usize][to as usize]
}
