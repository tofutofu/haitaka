// movegen
//use crate::*;
use super::*;

mod piece_moves;
pub use piece_moves::*;

// The private `commoner` module defines the private Commoner trait.
// This streamlines the implementation of move generation for all pieces apart from King.

mod commoner {
    use super::*;

    pub trait Commoner {
        const PIECE: Piece;

        fn pseudo_legals(color: Color, square: Square, blockers: BitBoard) -> BitBoard;
    }

    macro_rules! impl_moving_piece {
        ($square:ident,$color:ident,$blockers:ident; $($type:ident => $impl:expr),*) => {
            $(pub struct $type;

            impl Commoner for $type {
                const PIECE: Piece = Piece::$type;

                #[allow(unused_variables)]
                // allowing unused variables, since `blockers` is only used by sliders
                fn pseudo_legals($color: Color, $square: Square, $blockers: BitBoard) -> BitBoard {
                    $impl
                }
            })*
        };
    }

    impl_moving_piece! {
        square, color, blockers;
        Pawn => pawn_attacks(color, square),
        Lance => get_lance_moves(color, square, blockers),
        Knight => knight_attacks(color, square),
        Silver => silver_attacks(color, square),
        Gold => gold_attacks(color, square),
        Bishop => get_bishop_moves(color, square, blockers),
        Rook => get_rook_moves(color, square, blockers),
        Tokin => gold_attacks(color, square),
        PLance => gold_attacks(color, square),
        PSilver => gold_attacks(color, square),
        PKnight => gold_attacks(color, square),
        PBishop =>  get_bishop_moves(color, square, blockers) | gold_attacks(color, square),
        PRook => get_rook_moves(color, square, blockers) | silver_attacks(color, square)
    }
}

macro_rules! abort_if {
    ($($expr:expr),*) => {
        $(if $expr {
            return true;
        })*
    }
}

impl Board {
    // Target destination squares of board moves (other than by King).
    //
    // This function is only called when there is _at most one_ checker.
    // Its main purpose is to reduce the number of target squares when the
    // King is in check (and also to prevent illegal capture of one's own pieces
    // ...which actually sometimes has been observed in amateur tournaments...).
    //
    fn target_squares<const IN_CHECK: bool>(&self) -> BitBoard {
        let color = self.side_to_move();
        let targets = if IN_CHECK {
            // when in check, we must block the checker or capture it
            // (or the King must run, but this function is not used for King moves)
            let checker = self.checkers().next_square().unwrap();
            let our_king = self.king(color);
            get_between_rays(checker, our_king) | checker.bitboard()
        } else {
            BitBoard::FULL
        };
        targets & !self.colors(color)
    }

    // Similar to target_squares but for drop moves.
    //
    // In check, a drop can only be used to interpose. Otherwise, any empty square is ok.
    // Note that this doesn't exclude the forbidden drop ranks of Pawn, Lance and Knight.
    //
    fn target_drops<const IN_CHECK: bool>(&self) -> BitBoard {
        let color = self.side_to_move();

        if IN_CHECK {
            // when in check, we must block the checker
            let checker = self.checkers().next_square().unwrap();
            let our_king = self.king(color);
            get_between_rays(checker, our_king) & !self.occupied()
        } else {
            !self.occupied()
        }
    }

    // Board moves

    // Generate legal moves for all the "commoners" (all pieces except King).
    fn add_common_legals<
        P: commoner::Commoner,
        F: FnMut(PieceMoves) -> bool,
        const IN_CHECK: bool,
    >(
        &self,
        mask: BitBoard,
        listener: &mut F,
    ) -> bool {
        let color = self.side_to_move();
        let pieces = self.colored_pieces(color, P::PIECE) & mask;
        let pinned = self.pinned();
        let blockers = self.occupied();
        let target_squares = self.target_squares::<IN_CHECK>();

        for piece in pieces & !pinned {
            let moves = P::pseudo_legals(color, piece, blockers) & target_squares;
            if !moves.is_empty() {
                abort_if!(listener(PieceMoves::BoardMoves {
                    color,
                    piece: P::PIECE,
                    from: piece,
                    to: moves
                }));
            }
        }

        if !IN_CHECK && P::PIECE != Piece::Knight {
            // Pinned pieces (apart from Knight!) can still move along the attack ray between King and checker.
            // Only consider pinned pieces when not in check, since a pinned piece can never capture a checker.
            let our_king = self.king(color);
            for piece in pieces & pinned {
                let target_squares = target_squares & line_ray(our_king, piece);
                let moves = P::pseudo_legals(color, piece, blockers) & target_squares;
                if !moves.is_empty() {
                    abort_if!(listener(PieceMoves::BoardMoves {
                        color,
                        piece: P::PIECE,
                        from: piece,
                        to: moves
                    }));
                }
            }
        }
        false
    }

    // Is the King (of the side-to-move) safe on this square?
    //
    // This function seems a bit inefficient since it partially recomputes
    // opponent's attacks. But all those attacks have already been computed
    // on the preceding move (and could be precalculated for the first move),
    // so if they were cached this function could be optimized a lot.

    #[inline]
    fn king_safe_on(&self, square: Square) -> bool {
        macro_rules! lazy_and {
            ($lhs:expr, $rhs:expr) => {
                if $lhs.0 == 0 {
                    BitBoard::EMPTY
                } else {
                    $lhs & $rhs
                }
            };
        }

        macro_rules! short_circuit {
            ($($attackers:expr),*) => {
                $(if !$attackers.is_empty() {
                    return false;
                })*
                true
            }
        }

        let color = self.side_to_move();
        let their_pieces = self.colors(!color);
        let blockers =
            (self.occupied() ^ self.colored_pieces(color, Piece::King)) | square.bitboard();
        short_circuit! {
            gold_attacks(color, square) & their_pieces & (
                self.pieces(Piece::Gold) |
                self.pieces(Piece::Tokin) |
                self.pieces(Piece::PLance) |
                self.pieces(Piece::PKnight) |
                self.pieces(Piece::PSilver)
            ),
            silver_attacks(color, square) & their_pieces & self.pieces(Piece::Silver),
            knight_attacks(color, square) & their_pieces & self.pieces(Piece::Knight),
            pawn_attacks(color, square) & their_pieces & self.pieces(Piece::Pawn),
            lazy_and! {
                (self.pieces(Piece::Bishop) | self.pieces(Piece::PBishop)) & their_pieces,
                get_bishop_moves(color, square, blockers)
            },
            lazy_and! {
                (self.pieces(Piece::Rook) | self.pieces(Piece::PRook)) & their_pieces,
                get_rook_moves(color, square, blockers)
            },
            lazy_and! {
                self.pieces(Piece::Lance) & their_pieces,
                get_lance_moves(color, square, blockers)
            },
            king_attacks(color, square) & their_pieces & self.pieces(Piece::King)
        }
    }

    fn add_king_legals<F: FnMut(PieceMoves) -> bool, const IN_CHECK: bool>(
        &self,
        mask: BitBoard,
        listener: &mut F,
    ) -> bool {
        const PIECE: Piece = Piece::King;

        let color = self.side_to_move();
        let our_pieces = self.colors(color);
        let our_king = self.king(color);
        if !mask.has(our_king) {
            return false;
        }
        let mut moves = king_attacks(color, our_king) & !our_pieces;
        for to in moves {
            // removing unsafe squares should generally be more efficient than
            // adding safe squares to an originally empty bitboard, since
            // until the endgame most squares will be safe
            if !self.king_safe_on(to) {
                moves ^= to.bitboard();
            }
        }
        if !moves.is_empty() {
            abort_if!(listener(PieceMoves::BoardMoves {
                color,
                piece: PIECE,
                from: our_king,
                to: moves
            }));
        }
        false
    }

    fn add_all_legals<F: FnMut(PieceMoves) -> bool, const IN_CHECK: bool>(
        &self,
        mask: BitBoard,
        listener: &mut F,
    ) -> bool {
        abort_if! {
            self.add_common_legals::<commoner::Pawn, _, IN_CHECK>(mask, listener),
            self.add_common_legals::<commoner::Lance, _, IN_CHECK>(mask, listener),
            self.add_common_legals::<commoner::Knight, _, IN_CHECK>(mask, listener),
            self.add_common_legals::<commoner::Silver, _, IN_CHECK>(mask, listener),
            self.add_common_legals::<commoner::Gold, _, IN_CHECK>(mask, listener),
            self.add_common_legals::<commoner::Tokin, _, IN_CHECK>(mask, listener),
            self.add_common_legals::<commoner::PLance, _, IN_CHECK>(mask, listener),
            self.add_common_legals::<commoner::PKnight, _, IN_CHECK>(mask, listener),
            self.add_common_legals::<commoner::PSilver, _, IN_CHECK>(mask, listener),
            self.add_common_legals::<commoner::Bishop, _, IN_CHECK>(mask, listener),
            self.add_common_legals::<commoner::Rook, _, IN_CHECK>(mask, listener),
            self.add_common_legals::<commoner::PBishop, _, IN_CHECK>(mask, listener),
            self.add_common_legals::<commoner::PRook, _, IN_CHECK>(mask, listener),
            self.add_king_legals::<_, IN_CHECK>(mask, listener)
        }
        false
    }

    // Drops
    fn add_drops<P: commoner::Commoner, F: FnMut(PieceMoves) -> bool, const IN_CHECK: bool>(
        &self,
        listener: &mut F,
    ) -> bool {
        let color = self.side_to_move();
        let piece = P::PIECE;

        if self.inner.hand(color)[piece as usize] > 0 {
            let target_squares = self.target_drops::<IN_CHECK>();
            let permitted = drop_zone(color, piece);
            let to = target_squares & permitted;
            return listener(PieceMoves::Drops { color, piece, to });
        }
        false
    }

    fn add_all_drops<F: FnMut(PieceMoves) -> bool, const IN_CHECK: bool>(
        &self,
        listener: &mut F,
    ) -> bool {
        if self.is_hand_empty(self.side_to_move()) {
            return false;
        }
        abort_if! {
            self.add_drops::<commoner::Pawn, _, IN_CHECK>(listener),
            self.add_drops::<commoner::Lance, _, IN_CHECK>(listener),
            self.add_drops::<commoner::Knight, _, IN_CHECK>(listener),
            self.add_drops::<commoner::Silver, _, IN_CHECK>(listener),
            self.add_drops::<commoner::Gold, _, IN_CHECK>(listener),
            self.add_drops::<commoner::Rook, _, IN_CHECK>(listener),
            self.add_drops::<commoner::Bishop, _, IN_CHECK>(listener)
        }
        false
    }

    // Public API

    /// Is this move legal?
    pub fn is_legal(&self, mv: Move) -> bool {
        self.is_legal_drop(mv) || self.is_legal_board_move(mv)
    }

    /// Is this move a legal drop?
    pub fn is_legal_drop(&self, mv: Move) -> bool {
        if let Move::Drop { piece, to } = mv {
            let color = self.side_to_move();

            if piece == Piece::King
                || self.occupied().has(to)
                || no_fly_zone(color, piece).has(to)
                || (piece == Piece::Pawn && !self.pawn_drop_ok(color, to))
            {
                return false;
            }

            match self.checkers.len() {
                0 => return true,
                1 => return self.target_drops::<true>().has(to),
                _ => return false,
            }
        }
        false
    }

    /// Is this move a legal board move?
    pub fn is_legal_board_move(&self, mv: Move) -> bool {
        if let Move::BoardMove {
            from,
            to,
            promotion,
        } = mv
        {
            let color = self.side_to_move();
            let our_pieces = self.colors(color);

            if our_pieces.has(to) || !our_pieces.has(from) {
                return false;
            }

            let piece = match self.piece_on(from) {
                Some(piece) => piece,
                None => return false, // should be unreachable, but returning false seems safer
            };

            if piece == Piece::King {
                if promotion {
                    return false;
                }
                return self.king_is_legal(color, from, to);
            }

            if promotion {
                // `from` or `to` must be in the promotion zone
                let zone = prom_zone(color);
                if !(zone.has(to) || zone.has(from)) {
                    return false;
                }
            }

            // pinned piece are not allowed to move off the attack ray
            // but are allowed to move along that ray (when not in check)
            if self.pinned.has(from) && !line_ray(self.king(color), from).has(to) {
                return false;
            }

            // get permitted to-squares depending on checkers
            let target_squares: BitBoard = match self.checkers().len() {
                0 => self.target_squares::<false>(),
                1 => self.target_squares::<true>(),
                _ => return false, // if there are 2 checkers, King needed to move
            };

            // piece needs to move to a target square
            let attacks: BitBoard;
            match piece {
                Piece::Pawn => {
                    return (target_squares & pawn_attacks(color, from)).has(to);
                }
                Piece::Knight => {
                    return (target_squares & knight_attacks(color, from)).has(to);
                }
                Piece::Silver => {
                    return (target_squares & silver_attacks(color, from)).has(to);
                }
                Piece::Lance => {
                    attacks = lance_pseudo_attacks(color, from);
                    return (target_squares & attacks).has(to)
                        && (get_between_rays(from, to) & self.occupied()).is_empty();
                }
                Piece::Rook => {
                    attacks = rook_pseudo_attacks(from);
                    return (target_squares & attacks).has(to)
                        && (get_between_rays(from, to) & self.occupied()).is_empty();
                }
                Piece::PRook => {
                    attacks = rook_pseudo_attacks(from) | king_attacks(color, from);
                    return (target_squares & attacks).has(to)
                        && (get_between_rays(from, to) & self.occupied()).is_empty();
                }
                Piece::Bishop => {
                    attacks = bishop_pseudo_attacks(from);
                    return (target_squares & attacks).has(to)
                        && (get_between_rays(from, to) & self.occupied()).is_empty();
                }
                Piece::PBishop => {
                    attacks = bishop_pseudo_attacks(from) | king_attacks(color, from);
                    return (target_squares & attacks).has(to)
                        && (get_between_rays(from, to) & self.occupied()).is_empty();
                }
                Piece::King => {
                    return false; // cannot happen
                }
                _ => {
                    // Gold or promoted small pieces
                    return (target_squares & gold_attacks(color, from)).has(to);
                }
            }
        }
        false
    }

    fn king_is_legal(&self, color: Color, from: Square, to: Square) -> bool {
        if !(king_attacks(color, from) & !self.colors(color)).has(to) {
            false
        } else {
            self.king_safe_on(to)
        }
    }

    /// Generate all legal board moves and drops given a position in no particular order.
    ///
    /// To retrieve moves, a `listener` callback must be passed that receives [`PieceMoves`].
    ///
    /// The listener will be called max 1 time for the King of the side that is to move,
    /// max 2 times for every other piece on the board, and max 1 time for every piece type
    /// in hand. So, it will never be called more than 38 x 2 times.
    ///
    /// The listener can interrupt and stop move generation early by returning `true`.
    /// This function will then also return `true`. Otherwise, the function eventually
    /// returns `false` indicating that no more callbacks are to be expected.
    ///
    /// Note that the function signature requires `listener` to be passed as mutable (`mut`).
    /// This is because the `FnMut` trait allows the closure to mutate its captured environment,
    /// and passing it as mutable simplifies the implementation of move generation. The
    /// implementation may call the `listener` multiple times, but it will never actually
    /// modify the `listener` object itself.
    ///
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// let board = Board::startpos();
    /// let mut total_moves = 0;
    /// board.generate_moves(|moves| {
    ///     // Done this way for demonstration.
    ///     // Actual counting is best done in bulk with moves.len().
    ///     for _mv in moves {
    ///         total_moves += 1;
    ///     }
    ///     false
    /// });
    /// assert_eq!(total_moves, 30);
    /// ```
    pub fn generate_moves(&self, mut listener: impl FnMut(PieceMoves) -> bool) -> bool {
        abort_if! {
            self.generate_drops(&mut listener),
            self.generate_board_moves(&mut listener)
        }
        false
    }

    /// Generate all legal board moves.
    pub fn generate_board_moves(&self, listener: impl FnMut(PieceMoves) -> bool) -> bool {
        self.generate_board_moves_for(BitBoard::FULL, listener)
    }

    /// Generates moves for a subset of pieces.
    ///
    /// Argument `mask` is used to select the subset of pieces.
    ///
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// let board = Board::startpos();
    /// let pawns = board.pieces(Piece::Pawn);
    /// let mut pawn_moves = 0;
    /// board.generate_board_moves_for(pawns, |moves| {
    ///     // Done this way for demonstration.
    ///     // Actual counting is best done in bulk with moves.len().
    ///     for _mv in moves {
    ///         pawn_moves += 1;
    ///     }
    ///     false
    /// });
    /// assert_eq!(pawn_moves, 9);
    /// ```
    pub fn generate_board_moves_for(
        &self,
        mask: BitBoard,
        mut listener: impl FnMut(PieceMoves) -> bool,
    ) -> bool {
        match self.checkers().len() {
            0 => self.add_all_legals::<_, false>(mask, &mut listener),
            1 => self.add_all_legals::<_, true>(mask, &mut listener),
            _ => self.add_king_legals::<_, true>(mask, &mut listener),
        }
    }

    /// Generate all drops in no particular order.
    ///
    /// # Examples
    /// ```
    /// use sparrow::*;
    /// let sfen: & str = "lnsgk2nl/1r4gs1/p1pppp1pp/1p4p2/7P1/2P6/PP1PPPP1P/1SG4R1/LN2KGSNL b Bb 11";
    /// let board = Board::from_sfen(sfen).unwrap();
    /// assert_eq!(board.side_to_move(), Color::Black);
    /// let hand = board.hand(Color::Black);
    /// assert_eq!(hand[Piece::Bishop as usize], 1);
    /// let empty_squares = !board.occupied();
    /// let mut num_drops = 0;
    /// board.generate_drops(|moves| {
    ///     // should be able to drop the Bishop on every empty square
    ///     if let PieceMoves::Drops { color, piece, to } = moves {
    ///         assert_eq!(to, empty_squares);
    ///     } else {
    ///         assert!(false);
    ///     }
    ///     for mv in moves {
    ///         assert!(mv.is_drop());
    ///         num_drops += 1;
    ///     }
    ///     false
    /// });
    /// assert_eq!(num_drops, empty_squares.len());
    /// ```
    pub fn generate_drops(&self, mut listener: impl FnMut(PieceMoves) -> bool) -> bool {
        match self.checkers().len() {
            0 => self.add_all_drops::<_, false>(&mut listener),
            1 => self.add_all_drops::<_, true>(&mut listener),
            _ => false,
        }
    }

    /// Generate all drops for a particular piece.
    pub fn generate_drops_for(
        &self,
        piece: Piece,
        mut listener: impl FnMut(PieceMoves) -> bool,
    ) -> bool {
        let num_checkers = self.checkers.len();
        if num_checkers == 0 {
            match piece {
                Piece::Pawn => self.add_drops::<commoner::Pawn, _, false>(&mut listener),
                Piece::Lance => self.add_drops::<commoner::Lance, _, false>(&mut listener),
                Piece::Knight => self.add_drops::<commoner::Knight, _, false>(&mut listener),
                Piece::Silver => self.add_drops::<commoner::Silver, _, false>(&mut listener),
                Piece::Gold => self.add_drops::<commoner::Gold, _, false>(&mut listener),
                Piece::Rook => self.add_drops::<commoner::Rook, _, false>(&mut listener),
                Piece::Bishop => self.add_drops::<commoner::Bishop, _, false>(&mut listener),
                _ => false, // Other pieces cannot be dropped
            }
        } else if num_checkers == 1 {
            match piece {
                Piece::Pawn => self.add_drops::<commoner::Pawn, _, true>(&mut listener),
                Piece::Lance => self.add_drops::<commoner::Lance, _, true>(&mut listener),
                Piece::Knight => self.add_drops::<commoner::Knight, _, true>(&mut listener),
                Piece::Silver => self.add_drops::<commoner::Silver, _, true>(&mut listener),
                Piece::Gold => self.add_drops::<commoner::Gold, _, true>(&mut listener),
                Piece::Rook => self.add_drops::<commoner::Rook, _, true>(&mut listener),
                Piece::Bishop => self.add_drops::<commoner::Bishop, _, true>(&mut listener),
                _ => false, // Other pieces cannot be dropped
            }
        } else {
            // move than one checker, so no drops are legal
            false
        }
    }
}
