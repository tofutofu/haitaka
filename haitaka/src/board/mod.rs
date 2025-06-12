//! The Shogi [`Board`] representation and move generation functions
use crate::*;
use core::hash::{Hash, Hasher};
mod movegen;
mod parse;
mod validate;
mod zobrist;

pub use movegen::*;
pub use parse::*;
use zobrist::*;

/// The current state of the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameStatus {
    /// The game ended in a win for *the other side*
    /// (not the current side_to_move, but see also [`Board::status`])
    Won,
    /// The game ended in a draw
    Drawn,
    /// The game is still ongoing.
    Ongoing,
}

helpers::simple_error! {
    /// An error returned when the move played was illegal.
    pub struct IllegalMoveError = "The move played was illegal.";
}

/// SFEN string representing the start position
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
/// More in particular it also does not track the repetition status of positions.
/// Keeping track of that is a concern of a game-playing engine; the Board is only
/// concerned with representing, validating and modifying a position.
///
/// Before playing a move, `checkers` is the bitboard of all the opponent's pieces that
/// give check to our (side-to-move) King. The `pinned` bitboard has all our (side-to-move)
/// pieces that are pinned and can only move along an opponent slider's attack ray.
///
/// The Hash trait is supported by a custom hash function that uses the Zobrist board hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    inner: ZobristBoard,
    pinned: BitBoard,
    checkers: BitBoard,
    pawnless_files: [BitBoard; Color::NUM],
    move_number: u16,
}

/// Default only initializes an empty board.
///
/// This may be useful for setting up Tsume Shogi positions and for debugging.
/// Use [`Board::startpos`] to initialize the default start position.
impl Default for Board {
    fn default() -> Self {
        Self {
            inner: ZobristBoard::empty(),
            pinned: BitBoard::EMPTY,
            checkers: BitBoard::EMPTY,
            pawnless_files: [BitBoard::FULL; Color::NUM],
            move_number: 0,
        }
    }
}

impl Board {
    /// Get a board with the default start position.
    ///
    /// # Examples
    /// ```
    /// # use haitaka::*;
    /// let sfen: &str = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1";
    /// assert_eq!(Board::startpos(), sfen.parse().unwrap());
    /// ```
    pub fn startpos() -> Self {
        Self::from_sfen(SFEN_STARTPOS).unwrap()
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

    /// Does color have the given piece in hand?
    #[inline(always)]
    pub fn has_in_hand(&self, color: Color, piece: Piece) -> bool {
        self.inner.hand(color)[piece as usize] > 0
    }

    /// Return the number of pieces of the given piece type that color has in hand.
    #[inline(always)]
    pub fn num_in_hand(&self, color: Color, piece: Piece) -> u8 {
        self.inner.num_in_hand(color, piece)
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

    #[inline(always)]
    pub fn unchecked_put(&mut self, color: Color, piece: Piece, square: Square) {
        self.inner.xor_square(piece, color, square);
        if piece == Piece::Pawn {
            self.pawnless_files[color as usize] &= !square.file().bitboard();
        }
    }

    /// Get a [`BitBoard`] of all the pieces of the given piece type.
    #[inline(always)]
    pub const fn pieces(&self, piece: Piece) -> BitBoard {
        self.inner.pieces(piece)
    }

    /// Returns true if color has this particular piece on the board.
    #[inline(always)]
    pub fn has(&self, color: Color, piece: Piece) -> bool {
        !self.colored_pieces(color, piece).is_empty()
    }

    /// Get a [`BitBoard`] of all pieces in the current position that move like Gold.
    ///
    /// Note: This includes the Golds and all promoted pieces, including PRook and PBishop.
    /// Kings are not included.
    #[inline(always)]
    pub fn pseudo_golds(&self) -> BitBoard {
        self.inner.golds_and_promoted_pieces()
    }

    /// Get a [`BitBoard`] of all small pieces in the current position that move like Gold.
    ///
    /// This includes the Golds and all small promoted pieces, excluding PRook and PBishop.
    #[inline(always)]
    pub fn pseudo_tokins(&self) -> BitBoard {
        self.inner.pieces(Piece::Gold)
            | self.inner.pieces(Piece::Tokin)
            | self.inner.pieces(Piece::PSilver)
            | self.inner.pieces(Piece::PKnight)
            | self.inner.pieces(Piece::PLance)
    }

    /// Get a [`BitBoard`] of all pieces that move like Silver.
    ///
    /// This includes the PRook and PBishop. Kings are not included.
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
    /// # use haitaka::*;
    /// let board = Board::startpos();
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
    /// let black_pawns = board.colored_pieces(Color::Black, Piece::Pawn);
    /// assert_eq!(black_pawns, bitboard! {
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     X X X X X X X X X
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    /// });
    /// let white_lances = board.colored_pieces(Color::White, Piece::Lance);
    /// assert_eq!(white_lances, bitboard! {
    ///     X . . . . . . . X
    ///     . . . . . . . . .
    ///     . . . . . . . . .
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

    /// Get a [`BitBoard`] of all the sliders for color.
    ///
    /// # Examples
    ///
    /// ```
    /// use haitaka::*;
    /// let board = Board::startpos();
    /// assert_eq!(board.sliders(Color::White), bitboard! {
    ///     X . . . . . . . X
    ///     . X . . . . . X .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    /// });
    /// assert_eq!(board.sliders(Color::Black), bitboard! {
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . X . . . . . X .
    ///     X . . . . . . . X
    /// });
    /// ```
    #[inline(always)]
    pub fn sliders(&self, color: Color) -> BitBoard {
        (self.pieces(Piece::Lance)
            | self.pieces(Piece::Rook)
            | self.pieces(Piece::Bishop)
            | self.pieces(Piece::PRook)
            | self.pieces(Piece::PBishop))
            & self.colors(color)
    }

    /// Get a [`BitBoard`] of all the pieces on the board.
    /// # Examples
    /// ```
    /// # use haitaka::*;
    /// let board = Board::startpos();
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
    /// # use haitaka::*;
    /// let mut board = Board::startpos();
    /// assert_eq!(board.side_to_move(), Color::Black);
    /// board.play("2g2f".parse().unwrap());
    /// assert_eq!(board.side_to_move(), Color::White);
    /// board.play("3c3d".parse().unwrap());
    /// assert_eq!(board.side_to_move(), Color::Black);
    /// ```   
    #[inline(always)]
    pub fn side_to_move(&self) -> Color {
        self.inner.side_to_move()
    }

    /// Get the incrementally updated position hash.
    ///
    /// Does not include the move number.
    ///
    /// # Examples
    /// ```
    /// # use haitaka::*;
    /// let mut board = Board::startpos();
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
    ///
    /// Note that this counts pieces regardless of color!
    /// If there is a single piece, of any color, on an attack ray between our King
    /// (the King of the side to move) and their Rook, Bishop or Lance, it is counted
    /// as a 'pin'. This make it possible to simplify and optimize dealing with pins.
    ///
    /// # Examples
    ///
    /// ```
    /// use haitaka::*;
    /// let sfen: &str = "ln3gsn1/7kl/3+B1p1p1/p4s2p/2P6/P2B3PP/1PNP+rPP2/2G3SK1/L4G1NL b G3Prs3p 65";
    /// let mut board = Board::from_sfen(sfen).unwrap();
    /// // Since it's Black's turn, the Silver on D4 is not yet pinned
    /// assert_eq!(board.pinned(), BitBoard::EMPTY);
    /// let mv = Move::BoardMove { from: Square::C6, to: Square::A4, promotion: false };
    /// assert!(board.is_legal(mv));
    /// board.play(mv);
    /// // Now it's White's turn and the Silver on D4 should be pinned
    /// assert_eq!(board.pinned(), Square::D4.bitboard());
    /// ```
    #[inline(always)]
    pub fn pinned(&self) -> BitBoard {
        self.pinned
    }

    /// Get the pieces currently giving check.
    ///
    /// # Examples
    ///
    /// ```
    /// use haitaka::*;
    /// let sfen: &str = "ln3gsn1/7kl/3+B1p1p1/p4s2p/2P6/P2B3PP/1PNP+rPP2/2G3SK1/L4G1NL b G3Prs3p 65";
    /// let mut board = Board::from_sfen(sfen).unwrap();
    /// assert_eq!(board.checkers(), BitBoard::EMPTY);
    /// let mv = Move::BoardMove { from: Square::F6, to: Square::D4, promotion: false };
    /// board.play(mv);
    /// assert_eq!(board.checkers(), Square::D4.bitboard());
    ///
    /// // a rather absurd position with two checkers
    /// let sfen: &str = "ln2+r1r2/5s+Pkl/3+B1p1p1/p4B2p/2P6/P6PP/1PNP1P3/2G3SK1/L4G1NL w 2GSN3Ps3p 76";
    /// let mut board = Board::from_sfen(sfen).unwrap();
    /// assert_eq!(board.checkers(), Square::B3.bitboard() | Square::D4.bitboard());
    /// ```
    ///
    #[inline(always)]
    pub fn checkers(&self) -> BitBoard {
        self.checkers
    }

    /// Get the [move number].
    ///
    /// In Shogi, other than in International Chess, moves are always numbered
    /// by their "half-move number".
    ///
    /// # Examples
    ///
    /// ```
    /// # use haitaka::*;
    /// let mut board = Board::startpos();
    /// assert_eq!(board.move_number(), 1);
    /// board.play("2g2f".parse().unwrap());
    /// assert_eq!(board.move_number(), 2);
    /// board.play("8c8d".parse().unwrap());
    /// assert_eq!(board.move_number(), 3);
    /// board.play("2f2e".parse().unwrap());
    /// assert_eq!(board.move_number(), 4);
    /// board.play("8d8e".parse().unwrap());
    /// assert_eq!(board.move_number(), 5);
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
    /// # use haitaka::*;
    /// let mut board = Board::startpos();
    /// assert_eq!(board.move_number(), 1);
    /// board.set_move_number(20);
    /// assert_eq!(board.move_number(), 20);
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
    /// # use haitaka::*;
    /// let board = Board::startpos();
    /// assert_eq!(board.piece_on(Square::E5), None);
    /// assert_eq!(board.piece_on(Square::A5), Some(Piece::King));
    /// assert_eq!(board.piece_on(Square::I5), Some(Piece::King));
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
    /// # use haitaka::*;
    /// let board = Board::startpos();
    /// assert_eq!(board.color_on(Square::E5), None);
    /// assert_eq!(board.color_on(Square::A5), Some(Color::White));
    /// assert_eq!(board.color_on(Square::I5), Some(Color::Black));
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
    /// # use haitaka::*;
    /// let board = Board::startpos();
    /// let piece = ColoredPiece { piece: Piece::King, color: Color::Black };
    /// assert_eq!(board.color_on(Square::I5), Some(Color::Black));
    /// assert_eq!(board.piece_on(Square::I5), Some(Piece::King));
    /// assert_eq!(board.colored_piece_on(Square::I5), Some(piece));    
    /// ```
    pub fn colored_piece_on(&self, square: Square) -> Option<ColoredPiece> {
        if let Some(ref piece) = self.piece_on(square) {
            let color = self.color_on(square).unwrap();
            Some(ColoredPiece {
                piece: *piece,
                color,
            })
        } else {
            None
        }
    }

    /// Is a Pawn drop ok on the given square?
    ///
    /// This function return true if there is already a Pawn (of this color)
    /// somewhere on the file for this square. Its only purpose is to prevent
    /// dropping a double-pawn.
    ///
    /// # Examples
    /// ```
    /// use haitaka::*;
    /// let board = Board::default();
    /// for &square in Square::ALL.iter() {
    ///     let bok = board.pawn_drop_ok(Color::Black, square);
    ///     let wok = board.pawn_drop_ok(Color::White, square);
    ///     if square.rank() as usize != 0 {
    ///         assert!(bok);
    ///     }
    ///     if square.rank() as usize != 8 {
    ///         assert!(wok);
    ///     }
    /// }
    /// let board = Board::startpos();
    /// for &square in Square::ALL.iter() {
    ///     assert!(!board.pawn_drop_ok(Color::Black, square));
    /// }
    /// ```
    #[inline(always)]
    pub fn pawn_drop_ok(&self, color: Color, square: Square) -> bool {
        (self.colors(color) & self.pieces(Piece::Pawn) & square.file().bitboard()).is_empty()
    }

    /// Get the king square of the given side.
    ///
    /// # Panics
    /// This function panics if `color` has no King.
    ///
    /// # Examples
    /// ```
    /// # use haitaka::*;
    /// let board = Board::startpos();
    /// assert_eq!(board.king(Color::White), Square::A5);
    /// assert_eq!(board.king(Color::Black), Square::I5);
    /// ```
    #[inline(always)]
    pub fn king(&self, color: Color) -> Square {
        self.colored_pieces(color, Piece::King)
            .next_square()
            .expect("No king was found.")
    }

    /// Get the status of the game.
    ///
    /// This returns the current status of the game. If `GameStatus::Ongoing`
    /// then the game may still actually be a draw by Sennichite or Jishogi.
    /// If `GameStatus::Won` then the game is won by *the other side*, lost by
    /// the current `side_to_move`... unless the last move was an illegal
    /// checkmate by Pawn drop.
    ///
    /// Due to the rather complicated rules related to Sennichite and Jishogi
    /// the Board cannot always determine what the actual game status is. So
    /// this function has a pretty limited use. The final determination needs
    /// to be made by a game playing engine.
    ///
    /// The rules for winning and losing in Shogi are:
    ///
    /// - A player loses if they have no legal moves. This is either caused
    ///   by checkmate or (never seen in actual games) by not being able to
    ///   move any board piece without exposing the King to check (combined
    ///   with not having any pieces in hand). There is no "stalemate" in Shogi.
    /// - A player also loses if the same position reoccurs for the
    ///   fourth time while playing a sequence of consecutive checks. The player
    ///   who plays the checks loses.
    /// - A player loses in Jishogi (Double Entering King) if (1) the player has
    ///   less than 24 points, (2) both players have entered the King, and (3)
    ///   the inferior player has no chance of either checkmating the opponent or
    ///   increasing their number of points.
    /// - The game is a draw in Jishogi, if both players have at least 24 points.
    /// - The game is a draw by Sennichite, if the same position occurs for the
    ///   fourth time, and this was not caused by a sequence of continuous checks.
    ///
    pub fn status(&self) -> GameStatus {
        if self.generate_moves(|_| true) {
            GameStatus::Ongoing
        } else {
            // if we don't have any moves, it's a loss for us
            // (it doesn't matter if the position is checkmate)
            // ... unless ...
            // we were checkmated with an illegal Pawn drop,
            // in which case it's also a Win, but a Win for us
            // (this case can not be handled by `Board`)
            GameStatus::Won
        }
    }

    /// Check if two positions are equivalent.
    ///
    /// This differs from the [`Eq`] implementation in that it does not check the move number.
    /// This method can be used as a strict check for four-fold repetition or positions
    /// (Sennichite).
    ///
    /// # Examples
    ///
    /// ```
    /// use haitaka::*;
    /// let board1 = Board::startpos();
    /// let board2: Board = SFEN_STARTPOS.parse().unwrap();
    /// assert!(board1.same_position(&board2));
    /// ```
    pub fn same_position(&self, other: &Self) -> bool {
        self.hash() == other.hash() && self.inner.board_is_equal(&other.inner)
    }

    /// Check if this Board position dominates the other.
    ///
    /// A position dominates another position if the board positions are equal,
    /// but side-to-move has more pieces in hand (for each piece type) than in the
    /// other position. This is especially relevant in the endgame and in Tsume Shogi.
    /// If position P dominates Q and P does not have a forced win
    /// (for side-to-move), then Q will also not have a forced win. If Q can be solved
    /// in n moves, then P can be solved in at most n moves. If Q cannot be solved,
    /// then neither can P. So if P dominates Q, only Q needs to be searched to determine
    /// the status of both positions.
    ///
    /// When this Board dominates the other, this function returns Dominance::Dominates.
    /// When the reverse is the case, it returns Dominance::DominatedBy. If
    /// the positions are completely identical it returns Dominance::Equal. If they are
    /// incomparable, Dominance::Incomparable. Finally, when both board positions and
    /// hands are identical, but side-to-move is not, it returns Dominance::Sente which
    /// indicates that whoever is side-to-to-move has the advantage of the first move.
    ///
    pub fn dominates(&self, other: &Self) -> Dominance {
        self.inner.dominates(&other.inner)
    }

    /// Play a move while checking its legality.
    ///
    /// # Panics
    /// This panics if the move is illegal.
    /// See [`Board::try_play`] for a non-panicking variant.
    /// See [`Board::play_unchecked`] for a faster variant that allows illegal moves.
    ///
    /// # Examples
    /// ## Legal moves
    /// ```
    /// # use haitaka::*;
    /// let sfen: &str = "lnsgkgsnl/1r5b1/p1ppppppp/9/1p5P1/9/PPPPPPP1P/1B5R1/LNSGKGSNL b - 5";
    /// let mut board = Board::startpos();
    /// board.play("2g2f".parse().unwrap());
    /// board.play("8c8d".parse().unwrap());
    /// board.play("2f2e".parse().unwrap());
    /// board.play("8d8e".parse().unwrap());
    /// let sfen_out = format!("{}", board);
    /// assert_eq!(sfen_out, sfen);
    /// ```
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
    ///
    /// # Panics
    /// This may panic eventually if the move is illegal.
    ///
    /// Playing illegal moves corrupts the board state, which may cause further panics.
    /// See [`Board::play`] for a variant _guaranteed_ to panic immediately on illegal moves.
    ///
    /// # Examples
    /// ```
    /// # use haitaka::*;
    /// let mut board = Board::startpos();
    /// board.play_unchecked("2g2f".parse().unwrap());
    /// board.play_unchecked("8c8d".parse().unwrap());
    /// board.play_unchecked("2f2e".parse().unwrap());
    /// board.play_unchecked("8d8e".parse().unwrap());
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

            // update pawn_on_file
            if piece == Piece::Pawn {
                self.pawnless_files[color as usize] &= !to.file().bitboard();
            }

            // update checkers and pins
            if self.has(!color, Piece::King) {
                // opponent has a King - see which of our pieces are giving check
                self.update_checkers_and_pins(color, piece, to);
            } else {
                // opponent has no King (Tsume Shogi)
                self.checkers = BitBoard::EMPTY;
                self.pinned = BitBoard::EMPTY;
            }
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
            if let Some(capture) = self.piece_on(to) {
                // remove capture
                self.inner.xor_square(capture, !color, to);
                // take in hand
                self.inner.take_in_hand(color, capture.unpromote());

                // update pawn_on_file
                if capture == Piece::Pawn {
                    self.pawnless_files[!color as usize] |= to.file().bitboard();
                }
            }

            // lift piece up
            self.inner.xor_square(piece, color, from);

            // perhaps promote then drop piece
            let final_piece = if promotion { piece.promote() } else { piece };
            self.inner.xor_square(final_piece, color, to);

            // update pawn_on_file
            if piece == Piece::Pawn && promotion {
                self.pawnless_files[color as usize] |= to.file().bitboard();
            }

            // update checkers and pins (if the other side has a King)
            if self.has(!color, Piece::King) {
                // opponent has a King
                self.update_checkers_and_pins(color, final_piece, to);
            } else {
                // opponent has no King (Tsume Shogi)
                self.checkers = BitBoard::EMPTY;
                self.pinned = BitBoard::EMPTY;
            }
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
        debug_assert!(self.has(them, Piece::King));
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
            | Piece::Tokin
            | Piece::PLance
            | Piece::PKnight
            | Piece::PSilver
            | Piece::PBishop => {
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
        let rook_attacks = rook_pseudo_attacks(their_king) & rooks;
        let lance_attacks = lance_pseudo_attacks(them, their_king) & lances;

        let our_slider_attackers = our_pieces & (bishop_attacks | rook_attacks | lance_attacks);

        for attacker in our_slider_attackers {
            let between = get_between_rays(attacker, their_king) & occupied;
            match between.len() {
                0 => self.checkers |= attacker.bitboard(),
                1 => self.pinned |= between,
                _ => {}
            }
        }
    }

    /// Attempt to play a [null move](https://www.chessprogramming.org/Null_Move).
    /// Returns a new board if successful. Returns None if side-to-move is in check.
    ///
    /// A null move is a pass. A pass is not legal in Shogi (unless it means you resign).
    /// We can attempt a null move during the search, however, to see if this has an
    /// effect on the evaluation. If it doesn't significantly change the evaluation,
    /// we either already have a very bad position, or we are in a Zugzwang position
    /// (which is extremely rare in Shogi).
    /// If the King is in check, this function returns None. In that case a null
    /// move would make no sense (it would immediately lose).
    ///
    /// # Panics
    ///
    /// This function will panic if side-to-move has no King (Tsume Shogi).
    ///
    /// # Examples
    ///
    /// ```
    /// # use haitaka::*;
    /// let sfen1: &str = "lnsgkgsnl/1r5b1/p1ppppppp/9/1p5P1/9/PPPPPPP1P/1B5R1/LNSGKGSNL b - 5";
    /// let sfen2: &str = "lnsgkgsnl/1r5b1/p1ppppppp/9/1p5P1/9/PPPPPPP1P/1B5R1/LNSGKGSNL w - 6";
    /// let board1: Board = sfen1.parse().unwrap();
    /// let board2 = board1.null_move().unwrap();
    /// let sfen_out = format!("{}", board2);
    /// assert_eq!(sfen_out, sfen2);
    /// ```
    pub fn null_move(&self) -> Option<Board> {
        if self.checkers.is_empty() {
            let mut board = self.clone();
            let color = board.side_to_move();

            // update move number and switch side-to-move
            board.move_number += 1;
            board.inner.toggle_side_to_move();

            // we only need to update pinned
            board.pinned = BitBoard::EMPTY;
            let our_king = board.king(color);
            let them = board.colors(color);
            let their_attackers = them
                & (
                    bishop_pseudo_attacks(our_king) | rook_pseudo_attacks(our_king)
                    // already includes Lance attacks
                );
            let occ = board.occupied();
            for square in their_attackers {
                let between = get_between_rays(our_king, square) & occ;
                if between.len() == 1 {
                    board.pinned |= between;
                }
            }

            Some(board)
        } else {
            None
        }
    }
}

/// The Hash implementation for Board is using the Board Zobrist hash function.
impl Hash for Board {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash().hash(state)
    }
}
