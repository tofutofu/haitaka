use crate::*;

#[derive(Debug)]
struct ColorZobristConstants {
    pieces: [[u64; Square::NUM + 1]; Piece::NUM],
    hand: [[u64; 20]; Piece::NUM], // making room for counts
}

#[derive(Debug)]
struct ZobristConstants {
    color: [ColorZobristConstants; Color::NUM],
    move_toggle: u64,
}

const ZOBRIST: ZobristConstants = {
    // Simple Pcg64Mcg impl
    // Copied from cozy-chess - who copied it from the Rust `rand` crate.
    //
    // The initial seed is an odd number, seed > 2**127, with bit count 63.
    // The multiplier, mult > 2 ** 125 has bit count 65.
    //
    // The seed state is deliberately hard-coded to ensure consistency
    // in different program runs.
    //
    let mut state = 0x7369787465656E2062797465206E756Du128 | 1;
    macro_rules! rand {
        () => {{
            state = state.wrapping_mul(0x2360ED051FC65DA44385DF649FCCF645);
            let rot = (state >> 122) as u32;
            let xsl = ((state >> 64) as u64 ^ state as u64).rotate_right(rot);

            xsl
        }};
    }

    macro_rules! fill_array {
        ($array:ident: $expr:expr) => {{
            let mut i = 0;
            while i < $array.len() {
                $array[i] = $expr;
                i += 1;
            }
        }};
    }

    macro_rules! color_zobrist_constant {
        () => {{
            let mut pieces = [[0u64; Square::NUM + 1]; Piece::NUM];
            let mut hand = [[0u64; 20]; Piece::NUM];
            fill_array!(pieces: {
                let mut squares = [0; Square::NUM + 1];
                fill_array!(squares: rand!());
                squares
            });
            fill_array!(hand: {
                let mut counts = [0; 20];
                fill_array!(counts: rand!());
                counts
            });

            ColorZobristConstants {
                pieces,
                hand
            }
        }};
    }

    let white = color_zobrist_constant!();
    let black = color_zobrist_constant!();
    let move_toggle = rand!();

    ZobristConstants {
        color: [white, black],
        move_toggle,
    }
};

// This is Copy for performance reasons, since Copy guarantees a bit-for-bit copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ZobristBoard {
    //
    pieces: [BitBoard; Piece::NUM + 1], // piece type => bit map of board locations
    colors: [BitBoard; Color::NUM],     // color => bit map of board locations
    hands: [[u8; Piece::NUM]; Color::NUM], // color => [number of pieces in hand, indexed by piece type]
    side_to_move: Color,
    hash: u64,
}

impl ZobristBoard {
    #[inline(always)]
    pub fn empty() -> Self {
        Self {
            pieces: [BitBoard::EMPTY; Piece::NUM + 1],
            colors: [BitBoard::EMPTY; Color::NUM],
            hands: [[0; Piece::NUM]; Color::NUM],
            side_to_move: Color::Black,
            hash: 0,
        }
    }

    #[inline(always)]
    pub fn pieces(&self, piece: Piece) -> BitBoard {
        self.pieces[piece as usize]
    }

    #[inline(always)]
    pub fn golds_and_promoted_pieces(&self) -> BitBoard {
        // also includes King actually, which should be fine
        self.pieces[Piece::NUM]
    }

    #[inline(always)]
    pub fn colors(&self, color: Color) -> BitBoard {
        self.colors[color as usize]
    }

    #[inline(always)]
    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    #[inline(always)]
    pub fn hand(&self, color: Color) -> &[u8; Piece::NUM] {
        &self.hands[color as usize]
    }

    #[inline(always)]
    pub fn hands(&self) -> &[[u8; Piece::NUM]; Color::NUM] {
        &self.hands
    }

    #[inline(always)]
    pub fn unchecked_set_hand(&mut self, color: Color, piece: Piece, count: u8) {
        let old_count = self.hands[color as usize][piece as usize];
        self.hands[color as usize][piece as usize] = count;
        self.xor_hand(color, piece, old_count, count);
    }

    #[inline(always)]
    pub fn take_in_hand(&mut self, color: Color, piece: Piece) {
        let piece = piece.unpromote();
        let old_count = self.hands[color as usize][piece as usize];
        self.hands[color as usize][piece as usize] = old_count + 1;
        self.xor_hand(color, piece, old_count, old_count + 1);
    }

    /// Take from hand
    ///
    /// # Panics
    /// If hand doesn't contain this piece.
    ///
    #[inline(always)]
    pub fn take_from_hand(&mut self, color: Color, piece: Piece) {
        let piece = piece.unpromote();
        if let Some(new_count) = self.hands[color as usize][piece as usize].checked_sub(1) {
            self.hands[color as usize][piece as usize] = new_count;
            self.xor_hand(color, piece, new_count + 1, new_count);
        } else {
            panic!("Hand doesn't contain piece");
        }
    }

    #[inline(always)]
    pub fn is_hand_empty(&self, color: Color) -> bool {
        self.hands[color as usize].iter().all(|&count| count == 0)
    }

    #[inline(always)]
    pub fn hash(&self) -> u64 {
        self.hash
    }

    pub fn board_is_equal(&self, other: &Self) -> bool {
        self.side_to_move == other.side_to_move
            && self.pieces == other.pieces
            && self.colors == other.colors
            && self.hands == other.hands
    }

    // Update Zobrist hash for putting and removing a piece.
    #[inline(always)]
    pub fn xor_square(&mut self, piece: Piece, color: Color, square: Square) {
        let square_bb = square.bitboard();
        self.pieces[piece as usize] ^= square_bb; // toggles
        self.colors[color as usize] ^= square_bb; // toggles
        if piece as usize >= Piece::Gold as usize {
            self.pieces[Piece::NUM] ^= square_bb;
        }
        self.hash ^= ZOBRIST.color[color as usize].pieces[piece as usize][square as usize];
    }

    // Update Zobrist hash for dropping a piece or taking a piece in hand.
    #[inline(always)]
    fn xor_hand(&mut self, color: Color, piece: Piece, old_count: u8, new_count: u8) {
        debug_assert!(
            (old_count as usize) < ZOBRIST.color[color as usize].hand[piece as usize].len()
        );
        debug_assert!(
            (new_count as usize) < ZOBRIST.color[color as usize].hand[piece as usize].len()
        );
        self.hash ^= ZOBRIST.color[color as usize].hand[piece as usize][old_count as usize];
        self.hash ^= ZOBRIST.color[color as usize].hand[piece as usize][new_count as usize];
    }

    #[inline(always)]
    pub fn toggle_side_to_move(&mut self) {
        self.side_to_move = !self.side_to_move;
        self.hash ^= ZOBRIST.move_toggle;
    }
}

#[cfg(test)]
mod tests {
    use crate::Board;

    // TODO: Test some more edge cases

    #[test]
    fn zobrist_transpositions() {
        let board = Board::startpos();

        const MOVES: &[[[&str; 4]; 2]] = &[
            [
                ["2g2f", "8c8d", "7g7f", "3c3d"],
                ["7g7f", "8c8d", "2g2f", "3c3d"],
            ],
            [
                ["2g2f", "3c3d", "7g7f", "8c8d"],
                ["7g7f", "8c8d", "2g2f", "3c3d"],
            ],
            [
                ["5g5f", "8c8d", "2h5h", "3c3d"],
                ["5g5f", "3c3d", "2h5h", "8c8d"],
            ],
        ];
        for (i, [moves_a, moves_b]) in MOVES.iter().enumerate() {
            let mut board_a = board.clone();
            let mut board_b = board.clone();
            for mv in moves_a {
                board_a.play_unchecked(mv.parse().unwrap());
            }
            for mv in moves_b {
                board_b.play_unchecked(mv.parse().unwrap());
            }
            assert_eq!(board_a.hash(), board_b.hash(), "Test {}", i + 1);
        }
    }
}
