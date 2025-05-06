use crate::*;

macro_rules! soft_assert {
    ($expr:expr) => {
        if !$expr {
            return false;
        }
    };
}

impl Board {
    /// Canonical implementation of board validity. Used for debugging.
    #[cfg(test)]
    #[allow(dead_code)]
    pub(crate) fn validity_check(&self, for_tsume: bool) -> bool {
        soft_assert!(self.is_valid(for_tsume));
        soft_assert!(self.piece_counts_are_valid());
        soft_assert!(self.checkers_and_pins_are_valid());
        soft_assert!(self.move_number_is_valid());
        true
    }

    /// Check if the board position is valid without considering checkers and pins.
    ///
    /// This does not validate the checkers and pins, but does verify that the King
    /// of the side_to_move is not in check.
    /// If the `for_tsume` flag is set, we check the validity the position for a
    /// Tsume Shogi problem. In this case, we do not require the presence of the Black King.
    /// (Since there are Double-king Tsume a Black King may still be present but usually
    /// is not.)
    pub(super) fn is_valid(&self, for_tsume: bool) -> bool {
        // Piece bitboards should not overlap.
        let mut occupied = BitBoard::EMPTY;
        for &piece in &Piece::ALL {
            let pieces = self.pieces(piece);
            soft_assert!((pieces & occupied).is_empty());
            occupied |= pieces;
        }

        // Colors should not overlap
        let whites = self.colors(Color::White);
        let blacks = self.colors(Color::Black);
        soft_assert!((whites & blacks).is_empty());

        // Occupied should match
        soft_assert!(occupied == self.occupied());

        for &color in &Color::ALL {
            let pieces = self.colors(color);
            soft_assert!(pieces.len() <= 39);
            if !for_tsume || self.has(color, Piece::King) {
                soft_assert!((pieces & self.pieces(Piece::King)).len() == 1);
            }

            let pawns = pieces & self.pieces(Piece::Pawn);
            let knights = pieces & self.pieces(Piece::Knight);
            let lances = pieces & self.pieces(Piece::Lance);

            let badlands_pawns = no_fly_zone(color, Piece::Pawn);
            let badlands_knights = no_fly_zone(color, Piece::Knight);

            soft_assert!((pawns & badlands_pawns).is_empty());
            soft_assert!((lances & badlands_pawns).is_empty());
            soft_assert!((knights & badlands_knights).is_empty());
        }

        // make sure we have two Kings on board, unless this is a Tsume position
        if !for_tsume {
            soft_assert!(self.pieces(Piece::King).len() == 2);
        }

        // make sure that the Kings are not touching each other
        if !for_tsume || self.has(Color::Black, Piece::King) {
            let white_king_square = self.king(Color::White);
            let black_king_square = self.king(Color::Black);
            let white_king_moves = king_attacks(Color::White, white_king_square);
            soft_assert!(!white_king_moves.has(black_king_square));
        }

        // our_checkers are all our pieces giving check to the opponents King
        let (our_checkers, _) = self.calculate_checkers_and_pins(!self.side_to_move());

        // Opponent should not be in check while it's our turn (self.side_to_move)
        soft_assert!(our_checkers.is_empty());

        true
    }

    /// Are the piece counts valid?
    ///
    /// This checks whether or not the total number of pieces is correct,
    /// looking at all piece types except King, summing the number of pieces
    /// in both hands and on the board and comparing that to the expected number.
    /// In order to also support handicap games (without too much fuss), we
    /// only check that the piece count does not exceed the expected maximum.
    pub(super) fn piece_counts_are_valid(&self) -> bool {
        let &hands = self.hands();
        for index in 0..7 {
            let piece = Piece::index_const(index);
            debug_assert!(piece != Piece::King);

            let num_board_pieces = (self.pieces(piece) | self.pieces(piece.promote())).len() as u8;
            let max_num = Piece::MAX_HAND[index];
            let sum = hands[0][index] + hands[1][index] + num_board_pieces;

            soft_assert!(sum <= max_num);
        }
        true
    }

    /// Assign all remaining pieces to White's hand. Used in setting up Tsume Shogi positions.
    pub(super) fn piece_counts_make_valid(&mut self) {
        let &hands = self.hands();
        for index in 0..7 {
            let piece = Piece::index_const(index);
            let num = (self.pieces(piece) | self.pieces(piece.promote())).len() as u8;
            let sum = hands[0][index] + hands[1][index] + num;
            let missing = Piece::MAX_HAND[index] - sum;
            self.unchecked_set_hand(Color::White, piece, missing);
        }
    }

    pub(super) fn checkers_and_pins_are_valid(&self) -> bool {
        let (checkers, pinned) = self.calculate_checkers_and_pins(self.side_to_move());
        soft_assert!(self.checkers == checkers);
        soft_assert!(self.pinned() == pinned);
        soft_assert!(self.checkers.len() < 3);
        true
    }

    pub(super) fn move_number_is_valid(&self) -> bool {
        self.move_number > 0
    }

    /// Calculate checkers and pins for color.
    ///
    /// This return a pair of bitboards, `(checkers, pinned)`, where `checkers` is the bitboard
    /// of all the opponent's pieces that attack `color`'s King and `pinned` is the bitboard of
    /// all pieces (of any color) that block an opponent's slider from attacking `color`'s King,
    /// assuming there is only one such blocking piece.
    pub(super) fn calculate_checkers_and_pins(&self, color: Color) -> (BitBoard, BitBoard) {
        let mut checkers = BitBoard::EMPTY;
        let mut pinned = BitBoard::EMPTY;

        if !self.has(color, Piece::King) {
            return (checkers, pinned);
        }

        let our_king = self.king(color);
        let their_pieces = self.colors(!color);

        let bishops = self.pieces(Piece::Bishop) | self.pieces(Piece::PBishop);
        let rooks = self.pieces(Piece::Rook) | self.pieces(Piece::PRook);
        let lances = self.pieces(Piece::Lance);

        let bishop_attacks = bishop_pseudo_attacks(our_king) & bishops;
        let rook_attacks = rook_pseudo_attacks(our_king) & rooks;
        let lance_attacks = lance_pseudo_attacks(color, our_king) & lances;
        let their_slider_attackers = their_pieces & (bishop_attacks | rook_attacks | lance_attacks);

        let occupied = self.occupied();

        for attacker in their_slider_attackers {
            let between = get_between_rays(attacker, our_king) & occupied;
            match between.len() {
                0 => checkers |= attacker.bitboard(),
                1 => pinned |= between,
                _ => {}
            }
        }

        // Check non-sliders (including short-range of PRook and PBishop).
        // Note that the opponent King is tested separately!

        checkers |= pawn_attacks(color, our_king) & their_pieces & self.pieces(Piece::Pawn);
        checkers |= knight_attacks(color, our_king) & their_pieces & self.pieces(Piece::Knight);
        checkers |= silver_attacks(color, our_king) & their_pieces & self.pseudo_silvers();
        checkers |= gold_attacks(color, our_king) & their_pieces & self.pseudo_golds();

        (checkers, pinned)
    }
}
