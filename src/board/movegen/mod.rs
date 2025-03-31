// movegen
//use crate::*;
use super::*;

mod piece_moves;
pub use piece_moves::*;

impl Board {
    // Squares we can move to (with pieces other than the King).
    //
    // When in check, the checker must be captured or blocked.
    // Only called when there is at most one checker.
    //
    pub(crate) fn target_squares<const IN_CHECK: bool>(&self) -> BitBoard {
        let color = self.side_to_move();
        let targets = if IN_CHECK {
            let checker = self.checkers().next_square().unwrap();
            let our_king = self.king(color);
            get_between_rays(checker, our_king) | checker.bitboard()
        } else {
            BitBoard::FULL
        };
        targets & !self.colors(color)
    }
}
