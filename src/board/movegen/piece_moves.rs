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

/// A structure representing multiple moves for a piece on the board.
/// Iterate it to unpack its moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BoardMoves {
    pub color: Color,
    pub piece: Piece,
    pub from: Square,
    pub to: BitBoard,
}

impl BoardMoves {
    /// Get the number of generated to-squares.
    ///
    /// The BoardMovesIter will generate at least this number of moves,
    /// but may generate up to twice as many, depending on piece and position.
    ///
    pub fn len(&self) -> usize {
        self.to.len() as usize
    }

    /// Check if there are no [`Move`]s.
    pub fn is_empty(&self) -> bool {
        self.to.is_empty()
    }

    /// Check if this set of moves contains a given [`Move`].
    pub fn has(&self, mv: Move) -> bool {
        if let Move::BoardMove {
            from,
            to,
            promotion,
        } = mv
        {
            if self.from != from || !self.to.has(to) {
                return false;
            }
            match PromotionStatus::new(self.color, self.piece, from, to) {
                PromotionStatus::CannotPromote => !promotion,
                PromotionStatus::MustPromote => promotion,
                PromotionStatus::MayPromote => true, // Either is valid!
            }
        } else {
            false
        }
    }
}

/// Iterator over the moves in a [`BoardMoves`] instance.
/// The associated item is a [`Move`].
pub struct BoardMovesIter {
    moves: BoardMoves,
    to: Option<Square>,
    // promotion factor is 2 for promotable pieces, otherwise 1
    // this is used to calculate the upperbound of the size_hint
    promotion_factor: usize,
}

impl BoardMovesIter {
    /// Helper function to calculate the number of moves for a pawn.
    fn len_for_pawn(&self, color: Color, num_targets: usize) -> usize {
        let must_prom_zone = must_prom_zone(color, Piece::Pawn);
        let prom_zone = prom_zone(color);
        let to = self.moves.to;

        // If any destination square is in the must-promote zone, no promotions are possible
        if !(to & must_prom_zone).is_empty() {
            num_targets
        }
        // If the pawn is already in the promotion zone or can move into it, promotions are possible
        else if prom_zone.has(self.moves.from) || !(prom_zone & to).is_empty() {
            2 * num_targets
        }
        // Otherwise, no promotions are possible
        else {
            num_targets
        }
    }

    // Helper to calculate the number of moves for a lance.
    fn len_for_lance(&self, color: Color, num_targets: usize) -> usize {
        let must_prom_zone = must_prom_zone(color, Piece::Lance);
        let prom_zone = prom_zone(color);
        let to = self.moves.to;

        let m = (to & prom_zone).len();
        if m > 0 {
            let n = (to & must_prom_zone).len();
            let k = (to & prom_zone.not()).len();
            // m already includes n (if n > 0) so we need to subtract n
            return (2 * m - n + k) as usize;
        }
        num_targets
    }

    // Helper to calculate the number of moves for a knight.
    fn len_for_knight(&self, color: Color, num_targets: usize) -> usize {
        let must_prom_zone = must_prom_zone(color, Piece::Knight);
        let prom_zone = prom_zone(color);
        let to = self.moves.to;

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

impl IntoIterator for BoardMoves {
    type Item = Move;

    type IntoIter = BoardMovesIter;

    fn into_iter(self) -> Self::IntoIter {
        BoardMovesIter {
            moves: self,
            to: None,
            promotion_factor: if self.piece.is_promotable() { 2 } else { 1 },
        }
    }
}

impl Iterator for BoardMovesIter {
    type Item = Move;

    // Promotions (for a given (`from`, `to`) pair) are always returned first.
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let from = self.moves.from;

        if let Some(to) = self.to {
            // previously returned item was a promotion
            // now return the corresponding non-promotion

            self.moves.to ^= to.bitboard();
            self.to = None;

            return Some(Move::BoardMove {
                from,
                to,
                promotion: false,
            });
        }

        let color = self.moves.color;
        let piece = self.moves.piece;
        let from = self.moves.from;
        let to = self.moves.to.next_square()?;

        let promotion = match PromotionStatus::new(color, piece, from, to) {
            PromotionStatus::CannotPromote => {
                self.moves.to ^= to.bitboard(); // eats `to` bit
                false
            }
            PromotionStatus::MayPromote => {
                // sets `self.to` to be used for non-promotion in next step
                self.to = Some(to);
                true
            }
            PromotionStatus::MustPromote => {
                // eats `to` bit
                self.moves.to ^= to.bitboard();
                true
            }
        };

        Some(Move::BoardMove {
            from,
            to,
            promotion,
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining_moves = self.moves.to.len() as usize;
        let pending = if self.to.is_some() { 1 } else { 0 };

        let lo = remaining_moves + pending;
        let hi = self.promotion_factor * remaining_moves + pending;
        (lo, Some(hi))
    }
}

impl ExactSizeIterator for BoardMovesIter {
    fn len(&self) -> usize {
        let num_targets = self.moves.to.len() as usize;
        if !self.moves.piece.is_promotable() {
            // piece is either King, Gold, or already promoted
            num_targets
        } else {
            // piece could still promote
            let color = self.moves.color;
            let piece = self.moves.piece;

            match piece {
                Piece::Pawn => self.len_for_pawn(color, num_targets),
                Piece::Lance => self.len_for_lance(color, num_targets),
                Piece::Knight => self.len_for_knight(color, num_targets),
                _ => {
                    // Silver, Rook or Bishop
                    let from = self.moves.from;
                    let to = self.moves.to;
                    let zone = prom_zone(color);

                    if zone.has(from) {
                        // piece can promote on any move
                        2 * num_targets
                    } else {
                        // piece can promote on some moves
                        (2 * (zone & to).len() + (zone.not() & to).len()) as usize
                    }
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
        let mv = BoardMoves {
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
        let mv = BoardMoves {
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
        let mv = BoardMoves {
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
