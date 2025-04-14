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
    pub(crate) fn validity_check(&self) -> bool {
        soft_assert!(self.is_valid());
        soft_assert!(self.hands_are_valid());
        soft_assert!(self.checkers_and_pins_are_valid());
        soft_assert!(self.move_number_is_valid());
        true
    }

    /// Check if the board position is valid without considering checkers and pins.
    ///
    /// This does not validate the checkers and pins, but does verify that the King
    ///  of the side_to_move is not in check.
    pub(super) fn is_valid(&self) -> bool {
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
            soft_assert!((pieces & self.pieces(Piece::King)).len() == 1);

            let pawns = pieces & self.pieces(Piece::Pawn);
            let knights = pieces & self.pieces(Piece::Knight);
            let lances = pieces & self.pieces(Piece::Lance);

            let badlands_pawns = no_fly_zone(color, Piece::Pawn);
            let badlands_knights = no_fly_zone(color, Piece::Knight);

            soft_assert!((pawns & badlands_pawns).is_empty());
            soft_assert!((lances & badlands_pawns).is_empty());
            soft_assert!((knights & badlands_knights).is_empty());
        }

        // make sure that the Kings are not touching each other
        let white_king_square = self.king(Color::White);
        let black_king_square = self.king(Color::Black);
        let white_king_moves = king_attacks(Color::White, white_king_square);
        soft_assert!(!white_king_moves.has(black_king_square));

        // our_checkers are all our pieces giving check to the opponents King
        let (our_checkers, _) = self.calculate_checkers_and_pins(!self.side_to_move());

        // Opponent should not be in check while it's our turn (self.side_to_move)
        soft_assert!(our_checkers.is_empty());

        true
    }

    /// Are the counts of pieces in hand valid?
    pub(super) fn hands_are_valid(&self) -> bool {
        let &hands = self.hands();
        for index in 0..7 {
            let max_num = Piece::MAX_HAND[index];
            let sum = hands[0][index] + hands[1][index];
            soft_assert!(sum <= max_num);
        }
        true
    }

    pub(super) fn checkers_and_pins_are_valid(&self) -> bool {
        let (checkers, pinned) = self.calculate_checkers_and_pins(self.side_to_move());
        soft_assert!(self.checkers() == checkers);
        soft_assert!(self.pinned() == pinned);
        soft_assert!(self.checkers().len() < 3);
        true
    }

    pub(super) fn move_number_is_valid(&self) -> bool {
        self.move_number > 0
    }

    // TODO: Check how often this function is used.
    // 1. Could this also be optimized using the Qugiy trick?
    // 2. Could we just dispense with it alltogether -- at least during search --
    //    so just allow the king to be captured (game over)? We can during the search
    //    but it needs to be called at least whenever a move is finalized, so that
    //    when a move is actually made the King is not left in check. (This means
    //    that when no such move can be found, the program should resign.)
    //

    /// Calculate checkers and pins.
    ///
    /// # Panics
    /// This function panics if `color` has no King on the board.
    ///
    pub(super) fn calculate_checkers_and_pins(&self, color: Color) -> (BitBoard, BitBoard) {
        let mut checkers = BitBoard::EMPTY;
        let mut pinned = BitBoard::EMPTY;

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
                1 => pinned |= between, // no test that it's ours?
                _ => {}
            }
        }

        // Check non-sliders (including short-range of PRook and PBishop).
        // Note that the opponent King is checked separately!

        checkers |= pawn_attacks(color, our_king) & their_pieces & self.pieces(Piece::Pawn);
        checkers |= knight_attacks(color, our_king) & their_pieces & self.pieces(Piece::Knight);
        checkers |= silver_attacks(color, our_king) & their_pieces & self.pseudo_silvers();
        checkers |= gold_attacks(color, our_king) & their_pieces & self.pseudo_golds();

        (checkers, pinned)
    }
}
