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

// TODO: Keep simple status of "Drawn" or distinguish "Sennichite" and "Jishogi"?
//    /// The game ended in a draw by sennichite.
//    DrawnBySennichite,
//    /// The game ended in a draw by jishogi.
//    DrawnByJishogi,

// To detect sennichite, we need to detect repe


/// The current state of the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameStatus {
    /// The game ended in a win for side_to_
    Won,
    /// The game ended in a draw
    Drawn,
    /// The game is still ongoing.
    Ongoing,
}

// See YaneuraOu source/types.h
//
// https://en.wikipedia.org/wiki/Sennichite
// 
// "If the same game position occurs four times with the same player to move 
// and the same pieces in hand for each player, then the game ends in sennichite 
//      iff 
// the positions are not due to perpetual check. 
// (Perpetual check is an illegal move, which ends the game in a loss in tournament play.)"


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RepetitionStatus {
    /// No repetitions so far
    None,
    /// Win because of continuous checks with sennichite
    Win,   
    /// Loss becauss of continuus checks with sennichite
    Loss,
    /// Normal sennichite, without perpetual check
    Draw,
    /// YaneuraOu - on board same, but in hand better
    Superior,
    /// YaneuraOu - on board same, but in hand worse
    Inferior,
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

    pub fn calculate_checkers_and_pins(&self, _color: Color) -> (BitBoard, BitBoard) {
        todo!()
    }

    pub fn checkers_and_pins_are_valid(&self) -> bool {
        true
    }

    pub const fn move_number_is_valid(&self) -> bool {
        self.move_number > 0
    }



    pub fn hand(&self, color: Color) -> &[u8; Piece::NUM] {
        self.inner.hand(color)
    }

    pub fn is_hand_empty(&self, color: Color) -> bool {
        self.inner.is_hand_empty(color)
    }

    /////////

    #[inline(always)]
    pub fn pieces(&self, piece: Piece) -> BitBoard {
        self.inner.pieces(piece)
    }

    #[inline(always)]
    pub fn hands(&self) -> &[[u8; Piece::NUM]; Color::NUM] {
        self.inner.hands()
    }

    #[inline(always)]
    pub fn colors(&self, color: Color) -> BitBoard {
        self.inner.colors(color)
    }

    /// Get a [`BitBoard`] of all the pieces of a certain color and type.
    /// Shorthand for `board.colors(color) & board.pieces(piece)`.
    /// # Examples
    /// ```
    /// # use cozy_shogi::*;
    /// let board = Board::default();
    /// let white_pawns = board.colored_pieces(Color::White, Piece::Pawn);
    /// assert_eq!(white_pawns, bitboard! {
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     X X X X X X X X
    ///     . . . . . . . .
    /// });
    /// ```
    #[inline(always)]
    pub fn colored_pieces(&self, color: Color, piece: Piece) -> BitBoard {
        self.colors(color) & self.pieces(piece)
    }

    /// Get a [`BitBoard`] of all the pieces on the board.
    /// # Examples
    /// ```
    /// # use cozy_shogi::*;
    /// let board = Board::default();
    /// assert_eq!(board.occupied(), bitboard! {
    ///     X X X X X X X X
    ///     X X X X X X X X
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     X X X X X X X X
    ///     X X X X X X X X
    /// });
    /// ```
    #[inline(always)]
    pub fn occupied(&self) -> BitBoard {
        self.inner.colors(Color::White) | self.inner.colors(Color::Black)
    }

    /// Get the current side to move.
    /// # Examples
    /// ```
    /// # use cozy_shogi::*;
    /// let mut board = Board::default();
    /// assert_eq!(board.side_to_move(), Color::White);
    /// board.play("e2e4".parse().unwrap());
    /// assert_eq!(board.side_to_move(), Color::Black);
    /// ```    
    pub fn side_to_move(&self) -> Color {
        self.inner.side_to_move()
    }

    /// Get the incrementally updated position hash.
    /// 
    /// Does not include the move number.
    /// 
    /// # Examples
    /// ```
    /// # use cozy_shogi::*;
    /// let mut board = Board::default();
    /// board.play("e2e4".parse().unwrap());
    /// board.play("e7e5".parse().unwrap());
    /// board.play("e1e2".parse().unwrap());
    /// board.play("e8e7".parse().unwrap());
    /// let expected: Board = "rnbq1bnr/ppppkppp/8/4p3/4P3/8/PPPPKPPP/RNBQ1BNR w - - 2 3"
    ///    .parse().unwrap();
    /// assert_eq!(expected.hash(), board.hash());
    /// ```
    #[inline(always)]
    pub fn hash(&self) -> u64 {
        self.inner.hash()
    }

    /// Get the pinned pieces for the side to move.
    /// Note that this counts pieces regardless of color.
    /// This counts any piece preventing check on our king.
    /// # Examples
    /// ```
    /// # use cozy_shogi::*;
    /// let board: Board = "8/8/1q4k1/5p2/1n6/3B4/1KP3r1/8 w - - 0 1".parse().unwrap();
    /// assert_eq!(board.pinned(), bitboard! {
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . X . . . . . .
    ///     . . . . . . . .
    ///     . . X . . . . .
    ///     . . . . . . . .
    /// });
    /// ```
    #[inline(always)]
    pub fn pinned(&self) -> BitBoard {
        self.pinned
    }

    /// Get the pieces currently giving check.
    /// # Examples
    /// ```
    /// # use cozy_shogi::*;
    /// let mut board: Board = "1r4r1/pbpknp1p/1b3P2/8/8/B1PB1q2/P4PPP/3R2K1 w - - 0 22"
    ///     .parse().unwrap();
    /// assert_eq!(board.checkers(), BitBoard::EMPTY);
    /// board.play("d3f5".parse().unwrap());
    /// assert_eq!(board.checkers(), bitboard! {
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . X . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . X . . . .
    /// });
    /// ```
    #[inline(always)]
    pub fn checkers(&self) -> BitBoard {
        self.checkers
    }

    /// Get the [move number].
    /// 
    /// In Shogi, other than in International Chess, moves are always numbered 
    /// by their "half-move number" in Shogi. 
    ///
    /// # Examples
    /// ```
    /// # use cozy_shogi::*;
    /// let mut board = Board::default();
    /// // The fullmove number starts at one.
    /// assert_eq!(board.fullmove_number(), 1);
    /// board.play("e2e4".parse().unwrap());
    /// board.play("e7e5".parse().unwrap());
    /// board.play("e1e2".parse().unwrap());
    /// // 3 plies is 1.5 moves, which rounds down
    /// assert_eq!(board.fullmove_number(), 2);
    /// ```
    #[inline(always)]
    pub fn move_number(&self) -> u16 {
        self.move_number
    }

    /// Set the [move number]
    /// 
    /// # Panics
    /// This method panics if the argument is zero. The first move number is
    /// by convention 1.
    ///
    /// # Examples
    /// ```
    /// # use cozy_shogi::*;
    /// let mut board = Board::default();
    /// // The fullmove number starts at one.
    /// assert_eq!(board.fullmove_number(), 1);
    /// board.set_fullmove_number(2);
    /// assert_eq!(board.fullmove_number(), 2);
    /// ```
    #[inline(always)]
    pub fn set_move_number(&mut self, n: u16) {
        assert!(n > 0, "invalid move number {}", n);
        self.move_number = n;
    }

    /// Get the [`Piece`] on `square`, if there is one.
    /// 
    /// # Examples
    /// ```
    /// # use cozy_shogi::*;
    /// let board = Board::default();
    /// assert_eq!(board.piece_on(Square::A5), Some(Piece::King));
    /// ```
    #[inline(always)]
    pub fn piece_on(&self, square: Square) -> Option<Piece> {
        Some(Piece::King)
    }

    /// Get the [`Color`] of the piece on `square`, if there is one.
    /// 
    /// # Examples
    /// ```
    /// # use cozy_shogi::*;
    /// let board = Board::default();
    /// assert_eq!(board.color_on(Square::A5), Some(Color::White));
    /// ```
    #[inline(always)]
    pub fn color_on(&self, square: Square) -> Option<Color> {
        Some(Color::White)
    }

    /// Get the [`ColoredPiece`] on `square`, if there is one.
    /// 
    /// This is a convenience function, mainly useful in parsing of SFEN strings.
    /// 
    /// # Examples
    /// ```
    /// # use cozy_shogi::*;
    /// let board = Board::default();
    /// let cp = ColoredPiece { piece: Piece::King, color: Color::Black };
    /// assert_eq!(board.colored_piece_on(Square::I5), Some(Color::Black));    
    /// ```
    pub fn colored_piece_on(&self, square: Square) -> Option<ColoredPiece> {
        if let Some(piece) =  self.piece_on(square) {
            let color = self.color_on(square).unwrap();
            Some(ColoredPiece {piece, color})
        } else {            
            None
        }      
    }

    /// Get the king square of some side.
    /// # Examples
    /// ```
    /// # use cozy_shogi::*;
    /// let board = Board::default();
    /// assert_eq!(board.king(Color::White), Square::E1);
    /// ```
    #[inline(always)]
    pub fn king(&self, color: Color) -> Square {
        self.colored_pieces(color, Piece::King)
            .next_square()
            .expect("No king was found.")
    }

    // TODO: Review how this status function is used/can be improved.
    // I don't particularly like the cozy-chess impl.


    /// Get the status of the game.
    /// 
    /// Note that this game may still be drawn from threefold repetition.
    /// The game may also be drawn from insufficient material cases such
    /// as bare kings; This method does not detect such cases.
    /// If the game is won, the loser is the current side to move.
    /// 
    /// # Examples
    /// 
    /// ## Checkmate
    /// ```
    /// # use cozy_shogi::*;
    /// let mut board = Board::default();
    /// const MOVES: &[&str] = &[
    ///     "e2e4", "e7e5", "g1f3", "b8c6", "d2d4", "e5d4",
    ///     "f3d4", "f8c5", "c2c3", "d8f6", "d4c6", "f6f2"
    /// ];
    /// for mv in MOVES {
    ///     assert_eq!(board.status(), GameStatus::Ongoing);
    ///     board.play(mv.parse().unwrap());
    /// }
    /// assert_eq!(board.status(), GameStatus::Won);
    /// let winner = !board.side_to_move();
    /// assert_eq!(winner, Color::Black);
    /// ```
    /// ## Jishogi
    /// ```
    /// # use cozy_shogi::*;
    /// let mut board = Board::default();
    /// const MOVES: &[&str] = &[
    ///     "c2c4", "h7h5", "h2h4", "a7a5", "d1a4",
    ///     "a8a6", "a4a5", "a6h6", "a5c7", "f7f6",
    ///     "c7d7", "e8f7", "d7b7", "d8d3", "b7b8",
    ///     "d3h7", "b8c8", "f7g6", "c8e6"
    /// ];
    /// for mv in MOVES {
    ///     assert_eq!(board.status(), GameStatus::Ongoing);
    ///     board.play(mv.parse().unwrap());
    /// }
    /// assert_eq!(board.status(), GameStatus::Drawn);
    /// ```
    /// ## Sennichite
    /// 
    /// ```
    /// # use cozy_shogi::*;
    /// let mut board = Board::default();
    /// board.play("e2e4".parse().unwrap());
    /// board.play("e7e5".parse().unwrap());
    /// const MOVES: &[&str] = &["e1e2", "e8e7", "e2e1", "e7e8"];
    /// for mv in MOVES.iter().cycle().take(50 * 2) {
    ///     assert_eq!(board.status(), GameStatus::Ongoing);
    ///     board.play(mv.parse().unwrap());
    /// }
    /// assert_eq!(board.status(), GameStatus::Drawn);
    /// ```
    pub fn status(&self) -> GameStatus {
        /* 
        if self.generate_moves(|_| true) {
            if self.halfmove_clock() < 100 {
                GameStatus::Ongoing
            } else {
                GameStatus::Drawn
            }
        } else if self.checkers().is_empty() {
            GameStatus::Drawn
        } else {
            GameStatus::Won
        }
        */

        GameStatus::Ongoing
    }

    pub fn repetition_state(&self, ply: usize) -> RepetitionStatus {
        // TODO!
        RepetitionStatus::None
    }

    /// Check if two positions are equivalent.
    /// This differs from the [`Eq`] implementation in that it does not check the move number
    /// This method can be used as a strict check for four-fold repetition or positions.
    /// 
    pub fn same_position(&self, other: &Self) -> bool {
        self.hash() == other.hash() && self.inner.board_is_equal(&other.inner)
    }

    


}
