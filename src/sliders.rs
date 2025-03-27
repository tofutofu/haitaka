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
    loop { // this could be optimized, but it's not on the critical hot path
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
const fn get_slider_moves(square: Square, mut blockers: BitBoard, deltas: &[(i8, i8); 4]) -> BitBoard {
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
        Color::Black => -1
    };
    get_slider_moves(square, blockers, &[(0, dy), (0, 0), (0, 0), (0, 0)])
}


