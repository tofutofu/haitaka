use crate::*;

/// Simple structure to represent the promotability of a piece.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PromotionStatus {
    CannotPromote,
    MayPromote,
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

/// A compact enum representing all the moves for one particular piece
/// either on the board or in hand.
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
    },
}

impl PieceMoves {
    /// Get the number of generated to-squares.
    ///
    /// The PieceMovesIter will generate at least this number of moves,
    /// but may generate up to twice as many, depending on piece and position.
    ///
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
                match PromotionStatus::new(*color, *piece, *from, mv_to) {
                    PromotionStatus::CannotPromote => !promotion,
                    PromotionStatus::MustPromote => promotion,
                    PromotionStatus::MayPromote => true, // Either is valid!
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

    /// Helper function to calculate the number of board moves for a pawn.
    fn len_for_pawn(&self, color: Color, from: Square, to: BitBoard, num_targets: usize) -> usize {
        let must_prom_zone = must_prom_zone(color, Piece::Pawn);
        let prom_zone = prom_zone(color);

        // If any destination square is in the must-promote zone, no promotions are possible
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

    // Helper to calculate the number of board moves for a lance.
    fn len_for_lance(&self, color: Color, to: BitBoard, num_targets: usize) -> usize {
        let must_prom_zone = must_prom_zone(color, Piece::Lance);
        let prom_zone = prom_zone(color);

        let m = (to & prom_zone).len();
        if m > 0 {
            let n = (to & must_prom_zone).len();
            let k = (to & prom_zone.not()).len();
            // m already includes n (if n > 0) so we need to subtract n
            return (2 * m - n + k) as usize;
        }
        num_targets
    }

    // Helper to calculate the number of board moves for a knight.
    fn len_for_knight(&self, color: Color, to: BitBoard, num_targets: usize) -> usize {
        let must_prom_zone = must_prom_zone(color, Piece::Knight);
        let prom_zone = prom_zone(color);

        if (to & must_prom_zone).len() > 0 {
            // Knight must promote
            num_targets
        } else if (to & prom_zone).len() > 0 {
            // Knight may promote
            2 * num_targets
        } else {
            // no promotions
            num_targets
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
                let to_square = to.next_square()?;
                Some(Move::Drop {
                    piece: *piece,
                    to: to_square,
                })
            }
            // Handle board moves
            // Promotions (for a given (`from`, `to`) pair) are always returned first.
            PieceMoves::BoardMoves {
                color,
                piece,
                from,
                to,
            } => {
                let from = *from;

                if self.to.is_some() {
                    // previously returned item was a promotion
                    // now return the corresponding non-promotion

                    let to_square = self.to.unwrap();
                    self.to = None;

                    *to ^= to_square.bitboard();

                    return Some(Move::BoardMove {
                        from,
                        to: to_square,
                        promotion: false,
                    });
                }

                let to_square = to.next_square()?;

                let promotion = match PromotionStatus::new(*color, *piece, from, to_square) {
                    PromotionStatus::CannotPromote => {
                        *to ^= to_square.bitboard(); // eat `to` bit
                        false
                    }
                    PromotionStatus::MayPromote => {
                        // set `self.to` to make non-promotion in next step
                        self.to = Some(to_square);
                        true
                    }
                    PromotionStatus::MustPromote => {
                        *to ^= to_square.bitboard(); // eat `to` bit
                        true
                    }
                };

                Some(Move::BoardMove {
                    from,
                    to: to_square,
                    promotion,
                })
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
            } => {
                let num_targets = to.len() as usize;
                let pending_non_promotion = if self.to.is_some() { 1 } else { 0 };

                if !piece.is_promotable() {
                    // piece is either King, Gold, or already promoted
                    num_targets + pending_non_promotion
                } else {
                    // piece could still promote
                    let total_moves = match piece {
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

                    total_moves + pending_non_promotion
                }
            }
        }
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn len_handles_promotions() {
        let mv = PieceMoves {
            piece: Piece::Pawn,
            from: Square::A7,
            to: Square::A8.bitboard() | Square::B8.bitboard()
        };
        assert_eq!(mv.len(), 8);
        let mut iter = mv.into_iter();
        assert_eq!(iter.len(), 8);
        for len in (0..8).rev() {
            iter.next();
            assert_eq!(iter.len(), len);
        }
    }

    #[test]
    fn has_works() {
        let mv = PieceMoves {
            piece: Piece::King,
            from: Square::A7,
            to: get_king_moves(Square::A7)
        };
        assert!(!mv.has(Move {
            from: Square::A7,
            to: Square::A8,
            promotion: Some(Piece::Queen)
        }));
        assert!(mv.has(Move {
            from: Square::A7,
            to: Square::A8,
            promotion: None
        }));
    }

    #[test]
    fn has_handles_promotions() {
        let mv = PieceMoves {
            piece: Piece::Pawn,
            from: Square::A7,
            to: Square::A8.bitboard() | Square::B8.bitboard()
        };
        assert!(mv.has(Move {
            from: Square::A7,
            to: Square::A8,
            promotion: Some(Piece::Queen)
        }));
        assert!(!mv.has(Move {
            from: Square::A7,
            to: Square::A8,
            promotion: None
        }));
    }
}
*/
