use std::ops::BitAnd;

use crate::*;

/// Simple structure to represent the promotability of a piece.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PromotionStatus {
    Undecided,
    MayPromote,
    CannotPromote,
    MustPromote,
}

impl PromotionStatus {
    /// Check promotability of the given piece, moving from `from` to `to`.
    pub const fn new(color: Color, piece: Piece, from: Square, to: Square) -> Self {
        if piece.must_promote(color, to) {
            Self::MustPromote
        } else if piece.can_promote(color, to) || piece.can_promote(color, from) {
            Self::MayPromote
        } else {
            Self::CannotPromote
        }
    }
}

impl BitAnd for PromotionStatus {
    type Output = Self;

    // Implicit assumptions is that self and rhs are at compatible
    // so we don't try to add CannotPromote & MustPromote.
    // I'm not checking for this, but simply giving priority to `self`!
    fn bitand(self, rhs: Self) -> Self {
        match (self, rhs) {
            (PromotionStatus::Undecided, _) => rhs,
            (PromotionStatus::CannotPromote, _) => self,
            (PromotionStatus::MustPromote, _) => self,
            (PromotionStatus::MayPromote, PromotionStatus::Undecided) => self,
            (PromotionStatus::MayPromote, _) => rhs,
        }
    }
}

/// A compact enum representing all the moves for one particular piece.
///
/// Iterate over the PieceMoves instance to unpack the moves.
/// Note that the iterator will either return only [`Move::Drop`] or only
/// [`Move::BoardMove`] items depending on whether we iterate over drops or board moves.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceMoves {
    Drops {
        color: Color,
        piece: Piece,
        to: BitBoard,
    },
    BoardMoves {
        color: Color,
        piece: Piece,
        from: Square,
        to: BitBoard,
        prom_status: PromotionStatus,
    },
}

impl PieceMoves {
    /// Get the number of generated to-squares.
    ///
    /// The PieceMovesIter will generate _at least_ this number of moves,
    /// but may generate up to twice as many, depending on whether a piece may promote, must promote or can
    /// not promote. To get the accurate number of moves, do not use this function, but use
    /// `moves.into_iter()` on a PieceMoves instance.
    pub fn len(&self) -> usize {
        match self {
            PieceMoves::Drops { to, .. } | PieceMoves::BoardMoves { to, .. } => to.len() as usize,
        }
    }

    /// Check if there are no [`Move`]s.
    pub fn is_empty(&self) -> bool {
        match self {
            PieceMoves::Drops { to, .. } | PieceMoves::BoardMoves { to, .. } => to.is_empty(),
        }
    }

    /// Check if this set of moves contains a given [`Move`].
    /// The given move can either be a [`Move::Drop`] or [`Move::BoardMove`].
    pub fn has(&self, mv: Move) -> bool {
        match (self, mv) {
            // Handle BoardMoves
            (
                PieceMoves::BoardMoves {
                    color,
                    piece,
                    from,
                    to,
                    prom_status,
                },
                Move::BoardMove {
                    from: mv_from,
                    to: mv_to,
                    promotion,
                },
            ) => {
                if *from != mv_from || !to.has(mv_to) {
                    return false;
                }
                match *prom_status & PromotionStatus::new(*color, *piece, *from, mv_to) {
                    PromotionStatus::CannotPromote => !promotion,
                    PromotionStatus::MustPromote => promotion,
                    PromotionStatus::MayPromote => true,
                    _ => unreachable!(),
                }
            }
            // Handle Drops
            (
                PieceMoves::Drops { piece, to, .. },
                Move::Drop {
                    piece: mv_piece,
                    to: mv_to,
                },
            ) => *piece == mv_piece && to.has(mv_to),

            // If the variants don't match, return false
            _ => false,
        }
    }
}

/// Iterator over the moves in a [`PieceMoves`] instance.
/// The associated item is a [`Move`].
pub struct PieceMovesIter {
    moves: PieceMoves,
    // `to` is set to some square if we just returned a promotion move
    // and we want to return the corresponding non-promotion move on the next step;
    // for Drops this always remains None
    to: Option<Square>,
    // 'promotion_factor' is used to calculate the upperbound for the size_hint;
    // it is 2 for promotable pieces, otherwise 1;
    // for Drops it is always 1
    promotion_factor: usize,
}

impl PieceMovesIter {
    fn new(moves: PieceMoves) -> Self {
        let promotion_factor = match moves {
            PieceMoves::BoardMoves { piece, .. } if piece.is_promotable() => 2,
            _ => 1,
        };

        Self {
            moves,
            to: None,
            promotion_factor,
        }
    }

    // Helper function for a pawm to calculate the number of remaining board moves in `to`.
    // `to.len()` has already been calculated as `num_targets`.
    #[inline(always)]
    fn len_for_pawn(&self, color: Color, from: Square, to: BitBoard, num_targets: usize) -> usize {
        let must_prom_zone = must_prom_zone(color, Piece::Pawn);
        let prom_zone = prom_zone(color);

        // If the destination square (there can only be one)
        // is in the must-promote zone, no promotions are possible
        if !(to & must_prom_zone).is_empty() {
            num_targets
        }
        // If the pawn is already in the promotion zone or can move into it, promotions are possible
        else if prom_zone.has(from) || !(prom_zone & to).is_empty() {
            2 * num_targets
        }
        // Otherwise, no promotions are possible
        else {
            num_targets
        }
    }

    // Helper to calculate the number of board moves for a knight.
    #[inline(always)]
    fn len_for_knight(&self, color: Color, to: BitBoard, num_targets: usize) -> usize {
        let must_prom_zone = must_prom_zone(color, Piece::Knight);
        let prom_zone = prom_zone(color);

        if !(to & must_prom_zone).is_empty() {
            // Knight must promote
            num_targets
        } else if !(to & prom_zone).is_empty() {
            // Knight may promote
            2 * num_targets
        } else {
            // no promotions
            num_targets
        }
    }

    // Helper to calculate the number of board moves for a lance.
    #[inline(always)]
    fn len_for_lance(&self, color: Color, to: BitBoard, num_targets: usize) -> usize {
        debug_assert!(num_targets == to.len() as usize);
        let prom_zone = prom_zone(color);

        let m = (to & prom_zone).len() as usize;
        if m > 0 {
            // we have some possible promotions left
            let must_prom_zone = must_prom_zone(color, Piece::Lance);
            let n = (to & must_prom_zone).len() as usize;
            debug_assert!(n <= 1); // at most one required promotion on last rank
            debug_assert!(m >= n); // last rank is included in the promotion zone
            num_targets + m - n // add the extra promotion moves
        } else {
            num_targets // no promotions possible
        }
    }
}

impl IntoIterator for PieceMoves {
    type Item = Move;

    type IntoIter = PieceMovesIter;

    fn into_iter(self) -> Self::IntoIter {
        PieceMovesIter::new(self)
    }
}

impl Iterator for PieceMovesIter {
    type Item = Move;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.moves {
            // Handle drop moves
            PieceMoves::Drops { piece, to, .. } => {
                let to_square = to.next_square()?; // get it ...
                *to ^= to_square.bitboard(); // ... and reset it

                Some(Move::Drop {
                    piece: *piece,
                    to: to_square,
                })
            }
            // Handle board moves
            // When the piece can promote on a square, the promotion move is returned first.
            // In this case we cache the `to` square in `self.to` to generate the corresponding
            // non-promotion move on the next step. When a promotion is _required_, we do not
            // set `self.to`. So, `set.to` signals a pending non-promotion after a promotion.
            PieceMoves::BoardMoves {
                color,
                piece,
                from,
                to,
                prom_status,
            } => {
                let from = *from;

                if let Some(to_square) = self.to {
                    // previously returned item was a promotion
                    // now return the corresponding non-promotion

                    self.to = None;

                    Some(Move::BoardMove {
                        from,
                        to: to_square,
                        promotion: false,
                    })
                } else {
                    let to_square = to.next_square()?;
                    *to ^= to_square.bitboard(); // eat `to` bit

                    let promotion = match *prom_status
                        & PromotionStatus::new(*color, *piece, from, to_square)
                    {
                        PromotionStatus::CannotPromote => false,
                        PromotionStatus::MayPromote => {
                            // set `self.to` to generate non-promotion in next step
                            self.to = Some(to_square);
                            true
                        }
                        PromotionStatus::MustPromote => true,
                        _ => unreachable!(),
                    };

                    Some(Move::BoardMove {
                        from,
                        to: to_square,
                        promotion,
                    })
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.moves {
            PieceMoves::Drops { to, .. } => {
                let remaining_moves = to.len() as usize;
                (remaining_moves, Some(remaining_moves))
            }
            PieceMoves::BoardMoves { to, .. } => {
                let remaining_moves = to.len() as usize;
                let pending_non_promotion = if self.to.is_some() { 1 } else { 0 };

                let lo = remaining_moves + pending_non_promotion;
                let hi = self.promotion_factor * remaining_moves + pending_non_promotion;
                (lo, Some(hi))
            }
        }
    }
}

impl ExactSizeIterator for PieceMovesIter {
    fn len(&self) -> usize {
        match self.moves {
            PieceMoves::Drops { to, .. } => to.len() as usize,
            PieceMoves::BoardMoves {
                color,
                piece,
                from,
                to,
                prom_status,
            } => {
                let num_targets = to.len() as usize;
                let pending_non_promotion = if self.to.is_some() { 1 } else { 0 };

                if prom_status == PromotionStatus::CannotPromote
                    || prom_status == PromotionStatus::MustPromote
                {
                    debug_assert!(pending_non_promotion == 0);
                    num_targets
                } else {
                    // Undecided or MayPromote
                    let remaining_moves = match piece {
                        Piece::Pawn => self.len_for_pawn(color, from, to, num_targets),
                        Piece::Lance => self.len_for_lance(color, to, num_targets),
                        Piece::Knight => self.len_for_knight(color, to, num_targets),
                        _ => {
                            // Silver, Rook or Bishop
                            let zone = prom_zone(color);

                            if zone.has(from) {
                                // piece can always promote
                                2 * num_targets
                            } else {
                                // piece may sometimes promote
                                (2 * (zone & to).len() + (zone.not() & to).len()) as usize
                            }
                        }
                    };

                    remaining_moves + pending_non_promotion
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn len_works_with_non_promotions() {
        let mv = PieceMoves::BoardMoves {
            color: Color::Black,
            piece: Piece::Gold,
            from: Square::E5,
            to: gold_attacks(Color::Black, Square::E5),
            prom_status: PromotionStatus::CannotPromote,
        };
        assert_eq!(mv.len(), 6);
        let mut iter = mv.into_iter();
        assert_eq!(iter.len(), 6);

        for len in (0..6).rev() {
            iter.next();
            assert_eq!(iter.len(), len);
        }
    }

    // Multiple test cases for Lance, since Lance turned out to be the most
    // problematic piece; it took me several hours to debug a subtle bug the ExactSizeIterator.

    #[test]
    fn len_handles_promotions_black_lance() {
        let mv = PieceMoves::BoardMoves {
            color: Color::Black,
            piece: Piece::Lance,
            from: Square::I1,
            to: File::One.bitboard() ^ Square::I1.bitboard(),
            prom_status: PromotionStatus::Undecided,
        };
        assert_eq!(mv.len(), 8);

        let mut iter = mv.into_iter();
        assert_eq!(iter.len(), 10); // 3 promotions, but 1 required

        for len in (0..10).rev() {
            iter.next();
            assert_eq!(iter.len(), len);
        }

        let mv = PieceMoves::BoardMoves {
            color: Color::Black,
            piece: Piece::Lance,
            from: Square::I1,
            to: File::One.bitboard() ^ Square::I1.bitboard() ^ Square::A1.bitboard(),
            prom_status: PromotionStatus::Undecided,
        };
        assert_eq!(mv.len(), 7);

        let mut iter = mv.into_iter();
        assert_eq!(iter.len(), 9); // 2 promotion alternatives

        for len in (0..9).rev() {
            iter.next();
            assert_eq!(iter.len(), len);
        }
    }

    #[test]
    fn len_handles_promotions_white_lance() {
        let mv = PieceMoves::BoardMoves {
            color: Color::White,
            piece: Piece::Lance,
            from: Square::A1,
            to: File::One.bitboard() ^ Square::A1.bitboard(),
            prom_status: PromotionStatus::Undecided,
        };
        assert_eq!(mv.len(), 8);

        let mut iter = mv.into_iter();
        assert_eq!(iter.len(), 10); // 3 promotions, but 1 required

        for len in (0..10).rev() {
            iter.next();
            assert_eq!(iter.len(), len);
        }

        let mv = PieceMoves::BoardMoves {
            color: Color::White,
            piece: Piece::Lance,
            from: Square::A1,
            to: File::One.bitboard() ^ Square::A1.bitboard() ^ Square::I1.bitboard(),
            prom_status: PromotionStatus::Undecided,
        };
        assert_eq!(mv.len(), 7);

        let mut iter = mv.into_iter();
        assert_eq!(iter.len(), 9); // 2 promotion alternatives

        for len in (0..9).rev() {
            iter.next();
            assert_eq!(iter.len(), len);
        }
    }

    #[test]
    fn len_for_lance_handles_edge_cases() {
        // Case 1: Lance completely blocked
        let mv = PieceMoves::BoardMoves {
            color: Color::Black,
            piece: Piece::Lance,
            from: Square::I1,
            to: BitBoard::EMPTY, // No valid moves
            prom_status: PromotionStatus::Undecided,
        };
        assert_eq!(mv.len(), 0);
        let iter = mv.into_iter();
        assert_eq!(iter.len(), 0);

        // Case 2: Lance with one valid move in the must-promote zone
        let mv = PieceMoves::BoardMoves {
            color: Color::Black,
            piece: Piece::Lance,
            from: Square::B1,
            to: Square::A1.bitboard(), // Must-promote square
            prom_status: PromotionStatus::Undecided,
        };
        assert_eq!(mv.len(), 1);
        let mut iter = mv.into_iter();
        assert_eq!(iter.len(), 1);
        iter.next();
        assert_eq!(iter.len(), 0);

        // Case 3: Lance already inside promotion zone
        let mv = PieceMoves::BoardMoves {
            color: Color::Black,
            piece: Piece::Lance,
            from: Square::C1,
            to: Square::B1.bitboard() | Square::A1.bitboard(), // B1: may-promote, A1: must-promote
            prom_status: PromotionStatus::Undecided,
        };
        assert!(prom_zone(Color::Black).has(Square::C1));
        assert_eq!(mv.len(), 2);
        let mut iter = mv.into_iter();
        assert_eq!(iter.len(), 3);

        for len in (0..3).rev() {
            iter.next();
            assert_eq!(iter.len(), len);
        }
    }

    #[test]
    fn len_handles_promotions_rook() {
        let mv = PieceMoves::BoardMoves {
            color: Color::Black,
            piece: Piece::Rook,
            from: Square::E1,
            to: File::One.bitboard() ^ Square::E1.bitboard(),
            prom_status: PromotionStatus::Undecided,
        };
        assert_eq!(mv.len(), 8);

        let mut iter = mv.into_iter();
        assert_eq!(iter.len(), 11); // 3 promotions

        for len in (0..11).rev() {
            iter.next();
            assert_eq!(iter.len(), len);
        }

        let mv = PieceMoves::BoardMoves {
            color: Color::Black,
            piece: Piece::Rook,
            from: Square::C1,
            to: File::One.bitboard() ^ Square::C1.bitboard(),
            prom_status: PromotionStatus::Undecided,
        };
        assert_eq!(mv.len(), 8);

        let mut iter = mv.into_iter();
        assert_eq!(iter.len(), 16);

        for len in (0..16).rev() {
            iter.next();
            assert_eq!(iter.len(), len);
        }
    }

    #[test]
    fn len_handles_promotions_pawn() {
        let mv = PieceMoves::BoardMoves {
            color: Color::White,
            piece: Piece::Pawn,
            from: Square::F1,
            to: Square::G1.bitboard(),
            prom_status: PromotionStatus::Undecided,
        };
        assert_eq!(mv.len(), 1);

        let mut iter = mv.into_iter();
        assert_eq!(iter.len(), 2);

        for len in (0..2).rev() {
            iter.next();
            assert_eq!(iter.len(), len);
        }

        let mv = PieceMoves::BoardMoves {
            color: Color::Black,
            piece: Piece::Lance,
            from: Square::H1,
            to: Square::I1.bitboard(),
            prom_status: PromotionStatus::Undecided,
        };
        assert_eq!(mv.len(), 1);

        let mut iter = mv.into_iter();
        assert_eq!(iter.len(), 1); // promotion required

        iter.next();
        assert_eq!(iter.len(), 0);
    }

    #[test]
    fn has_works() {
        let mv = PieceMoves::BoardMoves {
            color: Color::White,
            piece: Piece::King,
            from: Square::A5,
            to: king_attacks(Color::White, Square::A5),
            prom_status: PromotionStatus::CannotPromote,
        };
        assert_eq!(mv.len(), 5); // remember, this is on an empty board

        assert!(mv.has(Move::BoardMove {
            from: Square::A5,
            to: Square::B4,
            promotion: false
        }));

        assert!(mv.has(Move::BoardMove {
            from: Square::A5,
            to: Square::B5,
            promotion: false
        }));

        assert!(mv.has(Move::BoardMove {
            from: Square::A5,
            to: Square::B6,
            promotion: false
        }));
    }

    #[test]
    fn has_handles_promotions() {
        let mv = PieceMoves::BoardMoves {
            color: Color::White,
            piece: Piece::Pawn,
            from: Square::F6,
            to: Square::G6.bitboard(),
            prom_status: PromotionStatus::Undecided,
        };
        assert_eq!(mv.len(), 1);

        assert!(mv.has(Move::BoardMove {
            from: Square::F6,
            to: Square::G6,
            promotion: false
        }));

        assert!(mv.has(Move::BoardMove {
            from: Square::F6,
            to: Square::G6,
            promotion: true
        }));
    }

    #[test]
    fn has_works_with_drops() {
        let mv = PieceMoves::Drops {
            color: Color::Black,
            piece: Piece::Rook,
            to: BitBoard::FULL,
        };

        assert_eq!(mv.len(), 81);

        for &square in Square::ALL.iter() {
            assert!(mv.has(Move::Drop {
                piece: Piece::Rook,
                to: square
            }));
        }
    }

    #[test]
    fn iter_undecided() {
        // promotion possible for every to square
        let mvs = PieceMoves::BoardMoves {
            color: Color::Black,
            piece: Piece::Silver,
            from: Square::C5,
            to: silver_attacks(Color::Black, Square::C5),
            prom_status: PromotionStatus::Undecided,
        };
        assert_eq!(mvs.len(), 5);
        assert_eq!(mvs.into_iter().len(), 10);

        // double-check in actual iteration
        let mut num_proms = 0;
        let mut num_non_proms = 0;
        for mv in mvs {
            if let Move::BoardMove {
                promotion: true, ..
            } = mv
            {
                num_proms += 1;
            } else {
                num_non_proms += 1;
            }
        }
        assert_eq!(num_non_proms, 5);
        assert_eq!(num_proms, 5);

        // promotion possible only on 3 squares
        let mvs = PieceMoves::BoardMoves {
            color: Color::Black,
            piece: Piece::Silver,
            from: Square::D5,
            to: silver_attacks(Color::Black, Square::D5),
            prom_status: PromotionStatus::Undecided,
        };
        assert_eq!(mvs.len(), 5);
        assert_eq!(mvs.into_iter().len(), 8);

        num_proms = 0;
        num_non_proms = 0;
        for mv in mvs {
            if let Move::BoardMove {
                promotion: true, ..
            } = mv
            {
                num_proms += 1;
            } else {
                num_non_proms += 1;
            }
        }
        assert_eq!(num_non_proms, 5);
        assert_eq!(num_proms, 3);
    }

    #[test]
    fn iter_cannot_promote() {
        // promotion  blocked by CannotPromote flag
        let mvs = PieceMoves::BoardMoves {
            color: Color::Black,
            piece: Piece::Silver,
            from: Square::C5,
            to: silver_attacks(Color::Black, Square::C5),
            prom_status: PromotionStatus::CannotPromote,
        };
        assert_eq!(mvs.len(), 5);
        assert_eq!(mvs.into_iter().len(), 5);

        let mut num_proms = 0;
        let mut num_non_proms = 0;
        for mv in mvs {
            if let Move::BoardMove {
                promotion: true, ..
            } = mv
            {
                num_proms += 1;
            } else {
                num_non_proms += 1;
            }
        }
        assert_eq!(num_non_proms, 5);
        assert_eq!(num_proms, 0);

        let mvs = PieceMoves::BoardMoves {
            color: Color::Black,
            piece: Piece::Silver,
            from: Square::D5,
            to: silver_attacks(Color::Black, Square::D5),
            prom_status: PromotionStatus::CannotPromote,
        };
        assert_eq!(mvs.len(), 5);
        assert_eq!(mvs.into_iter().len(), 5);

        let mut num_proms = 0;
        let mut num_non_proms = 0;
        for mv in mvs {
            if let Move::BoardMove {
                promotion: true, ..
            } = mv
            {
                num_proms += 1;
            } else {
                num_non_proms += 1;
            }
        }
        assert_eq!(num_non_proms, 5);
        assert_eq!(num_proms, 0);
    }

    #[test]
    fn iter_must_promote() {
        // promotion enforced by MustPromote flag
        let mvs = PieceMoves::BoardMoves {
            color: Color::Black,
            piece: Piece::Silver,
            from: Square::C5,
            to: silver_attacks(Color::Black, Square::C5),
            prom_status: PromotionStatus::MustPromote,
        };
        assert_eq!(mvs.len(), 5);
        assert_eq!(mvs.into_iter().len(), 5);

        let mut num_proms = 0;
        let mut num_non_proms = 0;
        for mv in mvs {
            if let Move::BoardMove {
                promotion: true, ..
            } = mv
            {
                num_proms += 1;
            } else {
                num_non_proms += 1;
            }
        }
        assert_eq!(num_non_proms, 0);
        assert_eq!(num_proms, 5);

        // only 3 promotion moves, and enforced by MustPromote flag
        let zone = prom_zone(Color::Black);
        let mvs = PieceMoves::BoardMoves {
            color: Color::Black,
            piece: Piece::Silver,
            from: Square::D5,
            to: silver_attacks(Color::Black, Square::D5) & zone,
            prom_status: PromotionStatus::MustPromote,
        };
        assert_eq!(mvs.len(), 3);
        assert_eq!(mvs.into_iter().len(), 3);

        num_proms = 0;
        num_non_proms = 0;
        for mv in mvs {
            if let Move::BoardMove {
                promotion: true, ..
            } = mv
            {
                num_proms += 1;
            } else {
                num_non_proms += 1;
            }
        }
        assert_eq!(num_non_proms, 0);
        assert_eq!(num_proms, 3);
    }
}
