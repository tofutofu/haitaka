use crate::*;

mod builder;
mod parse;
mod zobrist;

use builder::*;
use zobrist::*;
// use parse::*;

/*
mod movegen;
mod parse;
mod zobrist;
mod builder;
mod validate;

use zobrist::*;
pub use movegen::*;
pub use parse::*;
pub use builder::*;

*/

/// The current state of the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameStatus {
    /// The game ended in a win.
    Won,
    /// The game ended in a draw.
    Drawn,
    /// The game is still ongoing.
    Ongoing,
}

helpers::simple_error! {
    /// An error returned when the move played was illegal.
    pub struct IllegalMoveError = "The move played was illegal.";
}

/// A Shogi board.
///
/// This keeps about as much state as a SFEN string. It does not keep track of history.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Board {
    inner: ZobristBoard,
    pinned: BitBoard,
    checkers: BitBoard,
    move_number: u16,
}

impl Board {
    /// Get a board with the default start position.
    ///
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// let sfen_str = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPP/1B5R1/LNSGKGSNL w - 1";
    /// let start_pos: Board = sfen_str.parse().unwrap();
    /// assert_eq!(startpos, Board::default());
    /// ```
    pub fn startpos() -> Self {
        BoardBuilder::startpos().build().unwrap()
    }

    pub fn is_valid(&self) -> bool {
        true
    }

    pub fn side_to_move(&self) -> Color {
        Color::White
    }

    pub fn calculate_checkers_and_pins(&self, _color: Color) -> (BitBoard, BitBoard) {
        todo!()
    }

    pub fn checkers_and_pins_are_valid(&self) -> bool {
        true
    }

    pub const fn move_number_is_valid(&self) -> bool {
        self.move_number > 0
    }

    pub const fn move_number(&self) -> u16 {
        self.move_number
    }

    pub fn piece_on(&self, square: Square) -> Option<Piece> {
        Some(Piece::King)
    }

    pub fn color_on(&self, square: Square) -> Option<Color> {
        Some(Color::White)
    }

    pub fn colored_piece_on(&self, square: Square) -> Option<ColoredPiece> {
        Some(ColoredPiece {
            piece: Piece::King,
            color: Color::White,
        })
    }

    pub fn hand(&self, color: Color) -> &[u8; Piece::NUM] {
        self.inner.hand(color)
    }

    pub fn is_hand_empty(&self, color: Color) -> bool {
        self.inner.is_hand_empty(color)
    }

    #[inline(always)]
    pub fn colors(&self, color: Color) -> BitBoard {
        self.inner.colors(color)
    }

    #[inline(always)]
    pub fn pieces(&self, piece: Piece) -> BitBoard {
        self.inner.pieces(piece)
    }

    #[inline(always)]
    pub fn hands(&self) -> &[[u8; Piece::NUM]; Color::NUM] {
        self.inner.hands()
    }
}
