use super::*;

mod piece_moves;
pub use piece_moves::*;

#[cfg(test)]
mod tests;

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
    // King is in check (and to prevent illegal capture of one's own pieces
    // ...which actually sometimes is observed in amateur tournaments...).
    //
    fn target_squares<const IN_CHECK: bool>(&self) -> BitBoard {
        let color = self.side_to_move();
        let targets = if IN_CHECK {
            // when in check, we must block the checker or capture it
            // (or the King must run, but this function is not used for King moves)
            debug_assert!(self.checkers.len() == 1);
            let checker = self.checkers.next_square().unwrap();
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
        let open_squares = !self.occupied();

        if IN_CHECK {
            // when in check, we must block the checker
            let checkers = self.checkers & self.sliders(!color);
            if !checkers.is_empty() {
                debug_assert!(checkers.len() == 1);
                let checker = self.checkers.next_square().unwrap();
                let our_king = self.king(color);
                get_between_rays(checker, our_king) & open_squares
            } else {
                // check is not by a sliding piece, all drops are illegal!
                BitBoard::EMPTY
            }
        } else {
            open_squares
        }
    }

    // Board moves

    // Generate legal moves for all the "commoners" (all pieces except King).
    // `mask` is used to select from-squares
    fn add_common_legals<
        P: commoner::Commoner,
        F: FnMut(PieceMoves) -> bool,
        const IN_CHECK: bool,
    >(
        &self,
        mask: BitBoard,
        prom_status: PromotionStatus,
        listener: &mut F,
    ) -> bool {
        let target_squares = self.target_squares::<IN_CHECK>();

        if IN_CHECK && target_squares.is_empty() {
            return false;
        }

        let color = self.side_to_move();
        let pieces = self.colored_pieces(color, P::PIECE) & mask;
        let pinned = self.pinned;
        let blockers = self.occupied();

        for from in pieces & !pinned {
            let to = P::pseudo_legals(color, from, blockers) & target_squares;
            if !to.is_empty() {
                abort_if!(listener(PieceMoves::BoardMoves {
                    color,
                    piece: P::PIECE,
                    from,
                    to,
                    prom_status,
                }));
            }
        }

        if !IN_CHECK && P::PIECE != Piece::Knight && self.has(color, Piece::King) {
            // Pinned pieces (apart from Knight!) can still move along the attack ray between King and checker.
            // Only consider pinned pieces when not in check, since a pinned piece can never capture a checker.
            let our_king = self.king(color);
            for from in pieces & pinned {
                let to = P::pseudo_legals(color, from, blockers)
                    & line_ray(our_king, from)
                    & target_squares;

                if !to.is_empty() {
                    abort_if!(listener(PieceMoves::BoardMoves {
                        color,
                        piece: P::PIECE,
                        from,
                        to,
                        prom_status,
                    }));
                }
            }
        }
        false
    }

    #[allow(dead_code)]
    fn add_goldlike_legals<F: FnMut(PieceMoves) -> bool, const IN_CHECK: bool>(
        &self,
        mask: BitBoard,
        listener: &mut F,
    ) -> bool {
        let target_squares = self.target_squares::<IN_CHECK>();

        if IN_CHECK && target_squares.is_empty() {
            return false;
        }

        let color = self.side_to_move();
        let pieces = self.pseudo_tokins() & self.colors(color) & mask;
        let pinned = self.pinned;

        for from in pieces & !pinned {
            // Only goldlike pieces that are not pinned
            let to = gold_attacks(color, from) & target_squares;
            if !to.is_empty() {
                let piece = self.piece_on(from).unwrap();
                abort_if!(listener(PieceMoves::BoardMoves {
                    color,
                    piece,
                    from,
                    to,
                    prom_status: PromotionStatus::CannotPromote,
                }));
            }
        }

        if !IN_CHECK && self.has(color, Piece::King) {
            // Pinned gold-like pieces can still move along the attack ray between King and checker.
            // Only consider pinned pieces when not in check, since a pinned piece can never capture a checker!
            let our_king = self.king(color);
            for from in pieces & pinned {
                let to = gold_attacks(color, from) & line_ray(our_king, from) & target_squares;

                if !to.is_empty() {
                    let piece = self.piece_on(from).unwrap();
                    assert!((to & self.colors(color)).is_empty());
                    abort_if!(listener(PieceMoves::BoardMoves {
                        color,
                        piece,
                        from,
                        to,
                        prom_status: PromotionStatus::CannotPromote,
                    }));
                }
            }
        }
        false
    }

    // Is the King (of the side-to-move) safe on this square?
    //
    // This function seems inefficient since it recomputes the opponent's attacks.
    // But all those attacks have been computed on the preceding move, so if they
    // were cached this function could perhaps be optimized.
    // But there are two problems: (1) Move generation (generation of attacks) is
    // lazy. It can be interrupted by a listener, so that we don't generate all
    // possible moves. (2) Even if all moves (and attacks) are generated, those
    // become partially invalidated by actually making a move. Playing a move
    // invalidates the attacks of the moving piece and may either block sliders
    // or open up slider rays. So, it's not at all trivial to determine King
    // safety by efficient caching since this would require more work in
    // maintaining some cached data struct of 'attacks'. However...

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
        let kings = self.pieces(Piece::King);

        // simulate moving the King to the square (for slider attack generation)
        let blockers =
            (self.occupied() ^ self.colored_pieces(color, Piece::King)) | square.bitboard();

        // testing the sliders takes up about half of the test time;
        // using lazy_and improves throughput by about 17%
        short_circuit! {
            gold_attacks(color, square) & their_pieces & (self.pseudo_golds() | kings),
            silver_attacks(color, square) & their_pieces & (self.pseudo_silvers() | kings),
            knight_attacks(color, square) & their_pieces & self.pieces(Piece::Knight),
            pawn_attacks(color, square) & their_pieces & self.pieces(Piece::Pawn),
            lazy_and! {
                // by first filtering on pseudo attacks, this whole function becomes almost twice as fast
                bishop_pseudo_attacks(square) & (self.pieces(Piece::Bishop) | self.pieces(Piece::PBishop)) & their_pieces,
                get_bishop_moves(color, square, blockers)
            },
            lazy_and! {
                rook_pseudo_attacks(square) & (self.pieces(Piece::Rook) | self.pieces(Piece::PRook)) & their_pieces,
                get_rook_moves(color, square, blockers)
            },
            lazy_and! {
                lance_pseudo_attacks(color, square) & self.pieces(Piece::Lance) & their_pieces,
                get_lance_moves(color, square, blockers)
            }
        }
    }

    fn is_illegal_mate_by_pawn_drop(&self, to: Square) -> bool {
        debug_assert!(self.checkers.is_empty());

        let them = !self.side_to_move();
        let our_pawn_rank = to.rank() as usize;
        let their_king_rank = self.king(them).rank() as usize;

        if (them == Color::White && their_king_rank != our_pawn_rank - 1)
            || (them == Color::Black && their_king_rank != our_pawn_rank + 1)
        {
            return false;
        }

        // We know that our Pawn on `to` square attacks their King.
        //
        // (1) If to square is not attacked by them (apart from by their King), and
        // (2) to square is defended by at least one of ours, and
        // (3) King can not move (to square was the only remaining free square of the King)
        // then it is an illegal Pawn drop mate

        // For now, adding a slow version
        let mut board = self.clone();
        board.play_unchecked(Move::Drop {
            piece: Piece::Pawn,
            to,
        });

        // don't call generate_moves (which could cause recursion!)
        let mut has_legal_moves = false;
        board.generate_board_moves(|_| {
            has_legal_moves = true;
            true
        });

        !has_legal_moves
    }

    fn add_king_legals<F: FnMut(PieceMoves) -> bool, const IN_CHECK: bool>(
        &self,
        mask: BitBoard,
        listener: &mut F,
    ) -> bool {
        const PIECE: Piece = Piece::King;

        let color = self.side_to_move();
        if !self.has(color, Piece::King) {
            return false;
        }

        let our_pieces = self.colors(color);
        let our_king = self.king(color);
        if !mask.has(our_king) {
            return false;
        }
        let mut moves = king_attacks(color, our_king) & !our_pieces;
        for to in moves {
            // removing unsafe squares should generally be more efficient than
            // adding safe squares since (until the endgame) most squares are safe
            if !self.king_safe_on(to) {
                moves ^= to.bitboard();
            }
        }
        if !moves.is_empty() {
            abort_if!(listener(PieceMoves::BoardMoves {
                color,
                piece: PIECE,
                from: our_king,
                to: moves,
                prom_status: PromotionStatus::CannotPromote,
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
            self.add_common_legals::<commoner::Pawn, _, IN_CHECK>(mask, PromotionStatus::Undecided, listener),
            self.add_common_legals::<commoner::Lance, _, IN_CHECK>(mask, PromotionStatus::Undecided, listener),
            self.add_common_legals::<commoner::Knight, _, IN_CHECK>(mask, PromotionStatus::Undecided, listener),
            self.add_common_legals::<commoner::Silver, _, IN_CHECK>(mask, PromotionStatus::Undecided, listener),

            // doing all of the small gold-like pieces in one step consistently hurts the speed of move generation,
            // so for now I keep separate calls
            self.add_common_legals::<commoner::Gold, _, IN_CHECK>(mask, PromotionStatus::CannotPromote, listener),
            self.add_common_legals::<commoner::Tokin, _, IN_CHECK>(mask, PromotionStatus::CannotPromote, listener),
            self.add_common_legals::<commoner::PLance, _, IN_CHECK>(mask, PromotionStatus::CannotPromote, listener),
            self.add_common_legals::<commoner::PKnight, _, IN_CHECK>(mask, PromotionStatus::CannotPromote, listener),
            self.add_common_legals::<commoner::PSilver, _, IN_CHECK>(mask, PromotionStatus::CannotPromote, listener),

            self.add_common_legals::<commoner::Bishop, _, IN_CHECK>(mask, PromotionStatus::Undecided, listener),
            self.add_common_legals::<commoner::Rook, _, IN_CHECK>(mask, PromotionStatus::Undecided, listener),
            self.add_common_legals::<commoner::PBishop, _, IN_CHECK>(mask, PromotionStatus::CannotPromote, listener),
            self.add_common_legals::<commoner::PRook, _, IN_CHECK>(mask, PromotionStatus::CannotPromote, listener),
            self.add_king_legals::<_, IN_CHECK>(mask, listener)
        }
        false
    }

    // Drops
    fn add_drops<P: commoner::Commoner, F: FnMut(PieceMoves) -> bool, const IN_CHECK: bool>(
        &self,
        listener: &mut F,
        target_squares: BitBoard,
    ) -> bool {
        let color = self.side_to_move();
        let piece = P::PIECE;

        debug_assert!(!target_squares.is_empty());

        if self.has_in_hand(color, piece) {
            // limit targets to squares where piece may be dropped
            let mut to: BitBoard = target_squares & drop_zone(color, piece);

            if piece == Piece::Pawn {
                // prevent creating a double-pawn (nifu)
                to &= self.no_pawn_on_file[color as usize];
                if to.is_empty() {
                    return false;
                }
                // check that the drop doesn't cause illegal checkmate
                // note: if we're in check, this situation cannot occur!
                if !IN_CHECK {
                    let to_square = to.next_square().unwrap();
                    if self.is_illegal_mate_by_pawn_drop(to_square) {
                        to = to.rm(to_square);
                    }
                }
            }
            if to.is_empty() {
                return false;
            }

            return listener(PieceMoves::Drops { color, piece, to });
        }
        false
    }

    fn add_all_drops<F: FnMut(PieceMoves) -> bool, const IN_CHECK: bool>(
        &self,
        listener: &mut F,
        targets: BitBoard,
    ) -> bool {
        let color = self.side_to_move();
        if targets.is_empty() || self.is_hand_empty(color) {
            return false;
        }
        abort_if! {
            self.add_drops::<commoner::Pawn, _, IN_CHECK>(listener, targets),
            self.add_drops::<commoner::Lance, _, IN_CHECK>(listener, targets),
            self.add_drops::<commoner::Knight, _, IN_CHECK>(listener, targets),

            self.has_in_hand(color, Piece::Silver) &&
                listener(PieceMoves::Drops { color, piece: Piece::Silver, to: targets }),
            self.has_in_hand(color, Piece::Gold) &&
                listener(PieceMoves::Drops { color, piece: Piece::Gold, to: targets }),
            self.has_in_hand(color, Piece::Rook) &&
                listener(PieceMoves::Drops { color, piece: Piece::Rook, to: targets }),
            self.has_in_hand(color, Piece::Bishop) &&
                listener(PieceMoves::Drops { color, piece: Piece::Bishop, to: targets })
        }
        false
    }

    // Public API

    /// Is this move legal?
    #[inline(always)]
    pub fn is_legal(&self, mv: Move) -> bool {
        self.is_legal_board_move(mv) || self.is_legal_drop(mv)
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
                None => return false,
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
            } else if piece.must_promote(color, to) {
                return false;
            }

            // pinned piece are not allowed to move off the attack ray
            // but are allowed to move along that ray (when not in check)
            if self.pinned.has(from) && !line_ray(self.king(color), from).has(to) {
                return false;
            }

            // get permitted to-squares depending on checkers
            let target_squares: BitBoard = match self.checkers.len() {
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
    /// If you use the listener to update local state, then please remember that it may be
    /// called back several times by this function.
    ///
    /// The listener will be called max 1 time for the King of the side that is to move,
    /// max 2 times for every other piece on the board, and max 1 time for every piece type
    /// in hand. So, it will never be called more than 38 x 2 times.
    ///
    /// If the side_to_move is in check, and has no legal-moves, the listener will not be
    /// called. Normally this means the side_to_move has been checkmated. There is no stalemate
    /// in Shogi, however. If the side_to_move has no legal moves, they simply lose.
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
    ///
    /// ```
    /// # use haitaka::*;
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
        debug_assert!(self.inner.hash() != 0);
        self.generate_board_moves_for(BitBoard::FULL, listener)
    }

    /// Generates moves for a subset of pieces.
    ///
    /// Argument `mask` is used to select the subset of pieces.
    ///
    /// # Examples
    ///
    /// ```
    /// # use haitaka::*;
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
        match self.checkers.len() {
            0 => self.add_all_legals::<_, false>(mask, &mut listener),
            1 => self.add_all_legals::<_, true>(mask, &mut listener),
            _ => self.add_king_legals::<_, true>(mask, &mut listener),
        }
    }

    /// Generate all drops in no particular order.
    ///
    /// # Examples
    ///
    /// ```
    /// use haitaka::*;
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
        match self.checkers.len() {
            0 => {
                let targets = !self.occupied();
                self.add_all_drops::<_, false>(&mut listener, targets)
            }
            1 => {
                let targets = self.target_drops::<true>();
                self.add_all_drops::<_, true>(&mut listener, targets)
            }
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
            let dst = !self.occupied();
            match piece {
                Piece::Pawn => self.add_drops::<commoner::Pawn, _, false>(&mut listener, dst),
                Piece::Lance => self.add_drops::<commoner::Lance, _, false>(&mut listener, dst),
                Piece::Knight => self.add_drops::<commoner::Knight, _, false>(&mut listener, dst),
                Piece::Silver => self.add_drops::<commoner::Silver, _, false>(&mut listener, dst),
                Piece::Gold => self.add_drops::<commoner::Gold, _, false>(&mut listener, dst),
                Piece::Rook => self.add_drops::<commoner::Rook, _, false>(&mut listener, dst),
                Piece::Bishop => self.add_drops::<commoner::Bishop, _, false>(&mut listener, dst),
                _ => false, // Other pieces cannot be dropped
            }
        } else if num_checkers == 1 {
            let dst = self.target_drops::<true>();
            match piece {
                Piece::Pawn => self.add_drops::<commoner::Pawn, _, true>(&mut listener, dst),
                Piece::Lance => self.add_drops::<commoner::Lance, _, true>(&mut listener, dst),
                Piece::Knight => self.add_drops::<commoner::Knight, _, true>(&mut listener, dst),
                Piece::Silver => self.add_drops::<commoner::Silver, _, true>(&mut listener, dst),
                Piece::Gold => self.add_drops::<commoner::Gold, _, true>(&mut listener, dst),
                Piece::Rook => self.add_drops::<commoner::Rook, _, true>(&mut listener, dst),
                Piece::Bishop => self.add_drops::<commoner::Bishop, _, true>(&mut listener, dst),
                _ => false, // Other pieces cannot be dropped
            }
        } else {
            // there is more than one checker, so no drops are legal
            false
        }
    }

    /// Generate checks for side-to-move.
    pub fn generate_checks(&self, mut listener: impl FnMut(PieceMoves) -> bool) -> bool {
        let color = self.side_to_move();
        let their_color = !color;
        if !self.has(their_color, Piece::King) {
            return false;
        }

        let occ = self.occupied();
        let empty = !occ;

        let their_king = self.king(their_color);
        let their_ring = king_attacks(color, their_king);

        let rook_attacks = get_rook_moves(their_color, their_king, occ);
        let bishop_attacks = get_bishop_moves(their_color, their_king, occ);

        //
        // get all squares from which their King could be put in check
        //
        let mut attacks = [BitBoard::EMPTY; Piece::ALL.len()];
        for piece in Piece::ALL {
            attacks[piece as usize] = match piece {
                Piece::Pawn => pawn_attacks(their_color, their_king),
                Piece::Knight => knight_attacks(their_color, their_king),
                Piece::Silver => silver_attacks(their_color, their_king),
                Piece::Gold | Piece::Tokin | Piece::PLance | Piece::PKnight | Piece::PSilver => {
                    gold_attacks(their_color, their_king)
                }
                Piece::Lance => get_lance_moves(their_color, their_king, occ),
                Piece::Rook => rook_attacks,
                Piece::Bishop => bishop_attacks,
                Piece::PRook => rook_attacks | their_ring,
                Piece::PBishop => bishop_attacks | their_ring,
                _ => BitBoard::EMPTY,
            }
        }

        //
        // generate drops
        //
        let hand = self.hand(color);
        for index in 0..Piece::HAND_NUM {
            if hand[index] > 0 {
                let piece = Piece::index_const(index);
                let mut to = attacks[index] & empty;

                if piece == Piece::Pawn {
                    // avoid nifu
                    to &= self.no_pawn_on_file[color as usize];

                    // avoid illegal mate by pawn drop
                    let to_square = to.next_square().unwrap();
                    if self.is_illegal_mate_by_pawn_drop(to_square) {
                        to = to.rm(to_square);
                    }
                }

                if !to.is_empty() && listener(PieceMoves::Drops { color, piece, to }) {
                    return true;
                }
            }
        }

        //
        // generate checks with board moves
        //
        self.generate_board_moves(|mvs| {
            if let PieceMoves::BoardMoves {
                color,
                piece,
                from,
                to,
                prom_status,
            } = mvs
            {
                if prom_status == PromotionStatus::CannotPromote {
                    // only keep moves that attack the King
                    let to = to & attacks[piece as usize];
                    if !to.is_empty()
                        && listener(PieceMoves::BoardMoves {
                            color,
                            piece,
                            from,
                            to,
                            prom_status,
                        })
                    {
                        return true;
                    }
                } else {
                    debug_assert_eq!(prom_status, PromotionStatus::Undecided);
                    let checks = to & attacks[piece as usize];
                    // If we have moves with the unpromoted piece give check,
                    // return them, but make sure we do not promote.
                    // (These checks will never include squares on which piece MUST promote.
                    // since those square would not be in the attacks vector.)
                    if !to.is_empty()
                        && listener(PieceMoves::BoardMoves {
                            color,
                            piece,
                            from,
                            to: checks,
                            prom_status: PromotionStatus::CannotPromote,
                        })
                    {
                        return true;
                    }
                    // See if we also have promotions that give check
                    let zone = prom_zone(color);
                    let checks = if zone.has(from) {
                        to & attacks[piece.promote() as usize]
                    } else {
                        to & attacks[piece.promote() as usize] & zone
                    };
                    if checks.is_empty() {
                        return false;
                    }
                    // Returning the valid promotions that give check
                    return listener(PieceMoves::BoardMoves {
                        color,
                        piece,
                        from,
                        to: checks,
                        prom_status: PromotionStatus::MustPromote,
                    });
                }
            }
            false
        });

        false
    }
}
