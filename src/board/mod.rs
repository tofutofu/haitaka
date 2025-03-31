use crate::*;

mod movegen;
mod parse;
mod validate;
mod zobrist;

pub use movegen::*;
pub use parse::*;
use zobrist::*;

// TODO: Keep simple status of "Drawn" or distinguish "Sennichite" and "Jishogi"?
//    /// The game ended in a draw by sennichite.
//    DrawnBySennichite,
//    /// The game ended in a draw by jishogi.
//    DrawnByJishogi,

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

/// SFEN string representing the start position.
pub const SFEN_STARTPOS: &str = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1";

// TODO: In handicap games is white's first move numbered 1 or 2? For now, to be consistent, I label it '2'.

/// SFEN string for 6-piece handicap
pub const SFEN_6PIECE_HANDICAP: &str = "2sgkgs2/9/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 2";

/// SFEN string for 4-piece handicap
pub const SFEN_4PIECE_HANDICAP: &str =
    "1nsgkgsn1/9/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 2";

/// SFEN string for 2-piece handicap
pub const SFEN_2PIECE_HANDICAP: &str =
    "lnsgkgsnl/9/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 2";

/// A Shogi board.
///
/// This keeps about as much state as a SFEN string. It does not keep track of history.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Board {
    inner: ZobristBoard,
    pinned: BitBoard,
    checkers: BitBoard,
    move_number: u16, // TODO: change to usize?
}

impl Default for Board {
    fn default() -> Self {
        Self::from_sfen(SFEN_STARTPOS).unwrap()
    }
}

impl Board {
    /// Get a board with the default start position.
    ///
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// let sfen: &str = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1";
    /// assert_eq!(Board::startpos(), sfen.parse().unwrap());
    /// ```
    pub fn startpos() -> Self {
        Self::default()
    }

    /// Return a reference to the hand for color.
    pub fn hand(&self, color: Color) -> &[u8; Piece::NUM] {
        self.inner.hand(color)
    }

    /// Does color have no pieces in hand?
    #[inline(always)]
    pub fn is_hand_empty(&self, color: Color) -> bool {
        self.inner.is_hand_empty(color)
    }

    /// Set the count of a piece in hand for color.
    ///
    /// This function performs no checks in the validity of count!
    ///
    #[inline(always)]
    pub fn unchecked_set_hand(&mut self, color: Color, piece: Piece, count: u8) {
        self.inner.unchecked_set_hand(color, piece, count);
    }

    #[inline(always)]
    pub fn take_in_hand(&mut self, color: Color, piece: Piece) {
        self.inner.take_in_hand(color, piece);
    }

    /// Get a [`BitBoard`] of all the pieces of the given piece type.
    #[inline(always)]
    pub fn pieces(&self, piece: Piece) -> BitBoard {
        self.inner.pieces(piece)
    }

    // TODO: Review `pseudo_golds`
    // I think this will need to be called a _lot_, so it might be much better
    // to cache this.

    /// Get a [`BitBoard`] of all pieces that move like Gold
    /// (apart from the King).
    ///
    #[inline(always)]
    pub fn pseudo_golds(&self) -> BitBoard {
        self.inner.pieces(Piece::Gold)
            | self.inner.pieces(Piece::Tokin)
            | self.inner.pieces(Piece::PSilver)
            | self.inner.pieces(Piece::PKnight)
            | self.inner.pieces(Piece::PLance)
            | self.inner.pieces(Piece::PRook)
            | self.inner.pieces(Piece::PBishop)
    }

    /// Get a [`BitBoard`] of all pieces that move like Silver
    /// (apart from the King).
    ///
    #[inline(always)]
    pub fn pseudo_silvers(&self) -> BitBoard {
        self.inner.pieces(Piece::Silver)
            | self.inner.pieces(Piece::PRook)
            | self.inner.pieces(Piece::PBishop)
    }

    /// Get a reference to the hands array.
    #[inline(always)]
    pub fn hands(&self) -> &[[u8; Piece::NUM]; Color::NUM] {
        self.inner.hands()
    }

    /// Get a [`BitBoard`] of all the pieces of the given color.
    #[inline(always)]
    pub fn colors(&self, color: Color) -> BitBoard {
        self.inner.colors(color)
    }

    /// Get a [`BitBoard`] of all the pieces of a certain color and piece type.
    /// Shorthand for `board.colors(color) & board.pieces(piece)`.
    ///
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// let board = Board::default();
    /// let white_pawns = board.colored_pieces(Color::White, Piece::Pawn);
    /// assert_eq!(white_pawns, bitboard! {
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     X X X X X X X X X
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    /// });
    /// ```
    #[inline(always)]
    pub fn colored_pieces(&self, color: Color, piece: Piece) -> BitBoard {
        self.colors(color) & self.pieces(piece)
    }

    /// Get a [`BitBoard`] of all the pieces on the board.
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// let board = Board::default();
    /// assert_eq!(board.occupied(), bitboard! {
    ///     X X X X X X X X X
    ///     . X . . . . . X .
    ///     X X X X X X X X X
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     X X X X X X X X X
    ///     . X . . . . . X .
    ///     X X X X X X X X X
    /// });
    /// ```
    #[inline(always)]
    pub fn occupied(&self) -> BitBoard {
        self.inner.colors(Color::White) | self.inner.colors(Color::Black)
    }

    /// Get the current side to move.
    ///
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// let mut board = Board::default();
    /// assert_eq!(board.side_to_move(), Color::Black);
    /// board.play("2g2f".parse().unwrap());
    /// assert_eq!(board.side_to_move(), Color::White);
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
    /// # use sparrow::*;
    /// let mut board = Board::default();
    /// board.play("2g2f".parse().unwrap());
    /// board.play("8c8d".parse().unwrap());
    /// board.play("2f2e".parse().unwrap());
    /// board.play("8d8e".parse().unwrap());
    ///
    /// const SFEN: &str = "lnsgkgsnl/1r5b1/p1ppppppp/9/1p5P1/9/PPPPPPP1P/1B5R1/LNSGKGSNL b - 5";
    /// let expected: Board = Board::from_sfen(SFEN).unwrap();
    /// assert_eq!(expected.hash(), board.hash());
    /// ```
    #[inline(always)]
    pub fn hash(&self) -> u64 {
        self.inner.hash()
    }

    /// Get the pinned pieces for the side to move.
    /// Note that this counts pieces regardless of color.
    /// This counts any piece preventing check on our king.
    ///
    /// # Examples
    ///
    /// TODO
    ///
    #[inline(always)]
    pub fn pinned(&self) -> BitBoard {
        self.pinned
    }

    /// Get the pieces currently giving check.
    ///
    /// # Examples
    ///
    /// TODO
    ///
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
    /// # use sparrow::*;
    /// let mut board = Board::default();
    /// assert_eq!(board.move_number(), 1);
    /// board.play("7g7f".parse().unwrap());
    /// board.play("8c8d".parse().unwrap());
    /// board.play("7f7e".parse().unwrap());
    /// assert_eq!(board.move_number(), 4);
    /// ```
    #[inline(always)]
    pub fn move_number(&self) -> u16 {
        self.move_number
    }

    /// Set the [move number]
    ///
    /// # Panics
    /// This method panics if the argument is zero. The first move number in
    /// non-handicap games is by convention 1.
    ///
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// let mut board = Board::default();
    /// assert_eq!(board.move_number(), 1);
    /// board.set_move_number(20);
    /// assert_eq!(board.move_number(), 20);
    /// ```
    #[inline(always)]
    pub fn set_move_number(&mut self, n: u16) {
        assert!(n > 0, "invalid move number {}", n);
        self.move_number = n;
    }

    // TODO: Look into this:
    // The `piece_on`` function seems rather inefficient (slow)?
    // If we use an extra array, this can be replaced by simple lookup.
    // Question is that the extra array also needs to updated during
    // move/unmove so it's unclear if we'd gain any speed overall?

    /// Get the [`Piece`] on `square`, if there is one.
    ///
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// let board = Board::default();
    /// assert_eq!(board.piece_on(Square::A5), Some(Piece::King));
    /// assert_eq!(board.piece_on(Square::I5), Some(Piece::King));
    /// assert_eq!(board.piece_on(Square::E5), None);
    /// ```
    #[inline(always)]
    pub fn piece_on(&self, square: Square) -> Option<Piece> {
        Piece::ALL
            .iter()
            .copied()
            .find(|&p| self.pieces(p).has(square))
    }

    /// Get the [`Color`] of the piece on `square`, if there is one.
    ///
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// let board = Board::default();
    /// assert_eq!(board.color_on(Square::A5), Some(Color::White));
    /// assert_eq!(board.color_on(Square::I5), Some(Color::Black));
    /// assert_eq!(board.color_on(Square::E5), None);
    /// ```
    #[inline(always)]
    pub fn color_on(&self, square: Square) -> Option<Color> {
        if self.colors(Color::White).has(square) {
            Some(Color::White)
        } else if self.colors(Color::Black).has(square) {
            Some(Color::Black)
        } else {
            None
        }
    }

    /// Get the [`ColoredPiece`] on `square`, if there is one.
    ///
    /// This is a convenience function, mainly useful in parsing of SFEN strings.
    ///
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// let board = Board::default();
    /// let piece = ColoredPiece { piece: Piece::King, color: Color::Black };
    /// assert_eq!(board.color_on(Square::I5), Some(Color::Black));
    /// assert_eq!(board.piece_on(Square::I5), Some(Piece::King));
    /// assert_eq!(board.colored_piece_on(Square::I5), Some(piece));    
    /// ```
    pub fn colored_piece_on(&self, square: Square) -> Option<ColoredPiece> {
        if let Some(piece) = self.piece_on(square) {
            let color = self.color_on(square).unwrap();
            Some(ColoredPiece { piece, color })
        } else {
            None
        }
    }

    /// Is a Pawn drop ok on the given square?
    ///
    /// This does not check if the square itself is empty. It is only used
    /// to prevent dropping a double-pawn.
    ///
    #[inline(always)]
    pub fn pawn_drop_ok(&self, color: Color, square: Square) -> bool {
        (self.colors(color) & self.pieces(Piece::Pawn) & square.file().bitboard()).is_empty()
    }

    /// Get the king square of the given side.
    ///
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// let board = Board::default();
    /// assert_eq!(board.king(Color::White), Square::A5);
    /// assert_eq!(board.king(Color::Black), Square::I5);
    /// ```
    #[inline(always)]
    pub fn king(&self, color: Color) -> Square {
        self.colored_pieces(color, Piece::King)
            .next_square()
            .expect("No king was found.")
    }

    // TODO: Review how the `status` function is used/can be improved.
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
    ///
    /// ## Jishogi
    ///
    /// ## Sennichite Draw/Win/Loss
    ///
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

    pub fn repetition_state(&self, _ply: usize) -> RepetitionStatus {
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

    /// Play a move while checking its legality.
    ///
    /// # Panics
    ///
    /// This panics if the move is illegal.
    /// See [`Board::try_play`] for a non-panicking variant.
    /// See [`Board::play_unchecked`] for a faster variant that allows illegal moves.
    ///
    /// # Examples
    /// ## Legal moves
    /// ```
    /// # use sparrow::*;
    /// let mut board = Board::default();
    /// board.play("2g2f".parse().unwrap());
    /// board.play("8c8d".parse().unwrap());
    /// board.play("2f2e".parse().unwrap());
    /// board.play("8d8e".parse().unwrap());
    /// const SFEN: &str = "lnsgkgsnl/1r5b1/p1ppppppp/9/1p5P1/9/PPPPPPP1P/1B5R1/LNSGKGSNL b - 5";
    /// assert_eq!(format!("{}", board), SFEN);
    /// ```
    /// ## Illegal moves
    /// //```should_panic
    /// //# use sparrow::*;
    /// // let mut board = Board::default();
    /// // board.play("2g1g".parse().unwrap());
    /// //```
    pub fn play(&mut self, mv: Move) {
        assert!(self.try_play(mv).is_ok(), "Illegal move {}!", mv);
    }

    /// Non-panicking version of [`Board::play`].
    /// Tries to play a move, returning `Ok(())` on success.
    ///
    /// # Errors
    /// Errors with [`IllegalMoveError`] if the move was illegal.
    pub fn try_play(&mut self, mv: Move) -> Result<(), IllegalMoveError> {
        if !self.is_legal(mv) {
            return Err(IllegalMoveError);
        }
        self.play_unchecked(mv);
        Ok(())
    }

    /// Unchecked version of [`Board::play`].
    ///
    /// Use this method with caution. Only legal moves should ever be passed.
    /// Playing illegal moves may corrupt the board state and cause panics.
    /// (Even if it doesn't caused undefined behavior.)
    ///
    /// # Panics
    /// This may panic eventually if the move is illegal.
    ///
    /// Playing illegal moves may also corrupt the board state, which may cause further panics.
    /// See [`Board::play`] for a variant _guaranteed_ to panic immediately on illegal moves.
    ///
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// let mut board = Board::default();
    /// board.play_unchecked("P2g-2f".parse().unwrap());
    /// board.play_unchecked("P8c-8d".parse().unwrap());
    /// board.play_unchecked("P2f-2e".parse().unwrap());
    /// board.play_unchecked("P8d-8e".parse().unwrap());
    /// let expected: &str = "lnsgkgsnl/1r5b1/p1ppppppp/9/1p5P1/9/PPPPPPP1P/1B5R1/LNSGKGSNL b - 5";
    /// assert_eq!(format!("{}", board), expected);
    /// ```
    pub fn play_unchecked(&mut self, mv: Move) {
        let color = self.inner.side_to_move();

        if let Move::Drop { piece, to } = mv {
            // take piece out of hand
            self.inner.take_from_hand(color, piece);

            // drop the piece
            self.inner.xor_square(piece, color, to);

            // update checkers and pins
            self.update_checkers_and_pins(color, piece, to);
        } else if let Move::BoardMove {
            from,
            to,
            promotion,
        } = mv
        {
            // read piece from board
            let piece = self
                .piece_on(from)
                .expect("Missing piece on move's `from` square");

            // optional capture
            if let Some(capture) = self.piece_on(from) {
                // remove capture
                self.inner.xor_square(capture, !color, to);
                // take in hand
                self.inner.take_in_hand(color, capture.unpromote());
            }

            // lift piece up
            self.inner.xor_square(piece, color, from);

            // perhaps promote then drop piece
            let final_piece = if promotion { piece.promote() } else { piece };
            self.inner.xor_square(final_piece, color, to);

            // update checkers and pins
            self.update_checkers_and_pins(color, final_piece, to);
        }
        // update move_number
        self.move_number += 1;

        // update stm
        self.inner.toggle_side_to_move();
    }

    fn update_checkers_and_pins(&mut self, color: Color, piece: Piece, to: Square) {
        // reset pins and checkers
        self.pinned = BitBoard::EMPTY;
        self.checkers = BitBoard::EMPTY;

        // update for non-sliders
        let them = !color;
        let their_king = self.king(them);

        match piece {
            Piece::Pawn => {
                self.checkers |= pawn_attacks(them, their_king) & to.bitboard();
            }
            Piece::Knight => {
                self.checkers |= knight_attacks(them, their_king) & to.bitboard();
            }
            Piece::Silver | Piece::PRook => {
                self.checkers |= silver_attacks(them, their_king) & to.bitboard();
            }
            Piece::Gold
            | Piece::PBishop
            | Piece::Tokin
            | Piece::PLance
            | Piece::PKnight
            | Piece::PSilver => {
                self.checkers |= gold_attacks(them, their_king) & to.bitboard();
            }
            _ => {}
        }

        // update checkers and pins for sliders
        let our_pieces = self.colors(color);
        let occupied = self.occupied();

        let bishops = self.pieces(Piece::Bishop) | self.pieces(Piece::PBishop);
        let rooks = self.pieces(Piece::Rook) | self.pieces(Piece::PRook);
        let lances = self.pieces(Piece::Lance);

        let bishop_attacks = bishop_pseudo_attacks(their_king) & bishops;
        let rook_attacks = bishop_pseudo_attacks(their_king) & rooks;
        let lance_attacks = lance_pseudo_attacks(color, their_king) & lances;

        let our_slider_attackers = our_pieces & (bishop_attacks | rook_attacks | lance_attacks);

        for attacker in our_slider_attackers {
            let between = get_between_rays(attacker, their_king) & occupied;
            match between.len() {
                0 => self.checkers |= attacker.bitboard(),
                1 => self.pinned |= between, // note: this included pieces of both colors
                _ => {}
            }
        }
    }

    /// Attempt to play a [null move](https://www.chessprogramming.org/Null_Move),
    /// returning a new board if successful.
    ///
    /// A null move is a pass. A pass is not legal in Shogi (unless it means you resign).
    /// But during the search we can attempt a null move to see if this leaves the King
    /// in check. If the King is in check, this function returns None.
    ///
    /// # Examples
    ///
    /// TODO!
    ///
    /// ```
    pub fn null_move(&self) -> Option<Board> {
        None
        /*
        if self.checkers.is_empty() {
            let mut board = self.clone();
            board.move_number += 1;
            board.inner.toggle_side_to_move();

            // recalculate board.pinned
            board.pinned = BitBoard::EMPTY;

            let color = board.side_to_move();
            let our_king = board.king(color);
            let their_attackers = board.colors(!color) & (
                (get_bishop_rays(our_king) & (
                    board.pieces(Piece::Bishop) |
                    board.pieces(Piece::Queen)
                )) |
                (get_rook_rays(our_king) & (
                    board.pieces(Piece::Rook) |
                    board.pieces(Piece::Queen)
                ))
            );

            for square in their_attackers {
                let between = get_between_rays(square, our_king) & board.occupied();
                if between.len() == 1 {
                    board.pinned |= between;
                }
            }
            Some(board)
        } else {
            None
        }
        */
    }
}
