use crate::*;

use super::zobrist::ZobristBoard;

helpers::simple_error! {
    /// An error while building a board.
    pub enum BoardBuilderError {
        InvalidBoard = "The board is invalid.",
        InvalidMoveNumber = "The move number is invalid."
    }
}

/// A board builder to manipulate arbitrary boards.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BoardBuilder {
    /// The board state. Index by square to get the corresponding (piece, color).
    pub board: [Option<(Piece, Color)>; Square::NUM],
    /// The hands. Index by color and piece type index.
    pub hands: [[u8; Piece::NUM]; Color::NUM],
    /// The side to move.
    pub side_to_move: Color,
    /// The move number.
    pub move_number: u16
}

impl Default for BoardBuilder {
    fn default() -> Self {
        BoardBuilder::startpos()
    }
}

impl BoardBuilder {
    /// Get an empty builder. All fields are set to their empty values.
    /// 
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// let builder = BoardBuilder::empty();
    /// for &square in &Square::ALL {
    ///     assert!(builder.square(square).is_none());
    /// }
    /// ```
    pub fn empty() -> Self {
        Self {
            board: [None; Square::NUM],
            hands: [[0; Piece::NUM]; Color::NUM],
            side_to_move: Color::White,
            move_number: 1
        }
    }

    /// Get a builder set to the default start position.
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// let startpos = Board::default();
    /// let builder = BoardBuilder::default();
    /// assert_eq!(builder.build().unwrap(), startpos);
    /// ```
    pub fn startpos() -> Self {
        todo!()
    }

    /// Create a builder from a [`Board`].
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// let board = Board::default();
    /// let builder = BoardBuilder::from_board(&board);
    /// assert_eq!(builder.build().unwrap(), board);
    /// ```
    pub fn from_board(board: &Board) -> Self {
        let mut this = BoardBuilder::empty();
        for &color in &Color::ALL {
            let pieces = board.colors(color);
            for &piece in &Piece::ALL {
                let pieces = pieces & board.pieces(piece);
                for square in pieces {
                    *this.square_mut(square) = Some((piece, color));
                }
            }
        }
        this.hands = *board.hands(); // copies the hands
        this.side_to_move = board.side_to_move();
        this.move_number = board.move_number();
        this
    }

    /// Get a square on the board.
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// let builder = BoardBuilder::default();
    /// assert_eq!(builder.square(Square::A5), Some((Piece::King, Color::White)));
    /// assert_eq!(builder.square(Square::I5), Some((Piece::King, Color::Black)));
    /// ```
    pub fn square(&self, square: Square) -> Option<(Piece, Color)> {
        self.board[square as usize]
    }

    /// Mutably get a square on the board.
    /// 
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// let mut builder = BoardBuilder::default();
    /// *builder.square_mut(Square::A1) = Some((Piece::Rook, Color::White));
    /// assert_eq!(builder.square(Square::A1), Some((Piece::Rook, Color::White)));
    /// ```
    pub fn square_mut(&mut self, square: Square) -> &mut Option<(Piece, Color)> {
        &mut self.board[square as usize]
    }


    /// Build a [`Board`] from this builder.
    /// # Errors
    /// This will error if the current state is invalid.
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// let builder = BoardBuilder::default().build().unwrap();
    /// assert_eq!(builder, Board::default());
    /// ```
    pub fn build(&self) -> Result<Board, BoardBuilderError> {
        use BoardBuilderError::*;

        let mut board = Board {
            inner: ZobristBoard::empty(),
            pinned: BitBoard::EMPTY,
            checkers: BitBoard::EMPTY,
            move_number: 0
        };

        self.add_board          (&mut board).map_err(|_| InvalidBoard)?;
        self.add_fullmove_number(&mut board).map_err(|_| InvalidMoveNumber)?;
        
        Ok(board)
    }

    fn add_board(&self, board: &mut Board) -> Result<(), ()> {
        for &square in &Square::ALL {
            if let Some((piece, color)) = self.square(square) {
                board.inner.xor_square(piece, color, square);
            }
        }
        if self.side_to_move != board.side_to_move() {
            board.inner.toggle_side_to_move();
        }
        if !board.board_is_valid() {
            return Err(());
        }

        let (checkers, pinned) = board.calculate_checkers_and_pins(board.side_to_move());
        board.checkers = checkers;
        board.pinned = pinned;

        Ok(())
    }

    fn add_move_number(&self, board: &mut Board) -> Result<(), ()> {
        board.move_number = self.move_number;
        if !board.move_number_is_valid() {
            return Err(());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_board() {
        let positions = include_str!("test_data/valid.sfens");
        for fen in positions.lines() {
            let board = Board::from_fen(fen, true).unwrap();
            let builder = BoardBuilder::from_board(&board);
            assert_eq!(builder.build().unwrap(), board);
        }
    }
}
