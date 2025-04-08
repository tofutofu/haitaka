//! A simple [`Move`] enum to represent moves
//!
//! References:
//! - [wiki: Shogi Notation](https://en.wikipedia.org/wiki/Shogi_notation)
//! - [USI - Universal Shogi Interface](http://hgm.nubati.net/usi.html)
//!
use crate::*;
use core::str::FromStr;

/// A Shogi move.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Move {
    Drop {
        piece: Piece,
        to: Square,
    },
    BoardMove {
        from: Square,
        to: Square,
        promotion: bool,
    },
}

crate::helpers::simple_error! {
    /// The value was not a valid [`Move`].
    pub enum MoveParseError {
        InvalidPiece = "Invalid piece",
        InvalidSquare = "Invalid square",
        InvalidFormat = "Invalid move format",
        ExtraCharacters = "Extra characters found"
    }
}

impl Move {
    /// Is this move a promotion?
    #[inline(always)]
    pub const fn is_promotion(&self) -> bool {
        match self {
            Move::BoardMove { promotion, .. } => *promotion,
            Move::Drop { .. } => false,
        }
    }

    /// Is this move a drop?
    #[inline(always)]
    pub const fn is_drop(&self) -> bool {
        matches!(self, Move::Drop { .. })
    }

    /// Is this move a board move?
    #[inline(always)]
    pub const fn is_board_move(&self) -> bool {
        !self.is_drop()
    }

    /// Get the piece involved in the move.
    ///
    /// For `Move::Drop`, this directly returns the piece being dropped (wrapped in Some).
    /// For `Move::BoardMove`, this requires additional context (the board state)
    /// to infer the piece, so it returns `None` by default.
    pub fn piece(&self) -> Option<Piece> {
        match self {
            Move::Drop { piece, .. } => Some(*piece),
            Move::BoardMove { .. } => None,
        }
    }

    /*
    /// Get the piece involved in the move, using the board state for context.
    pub fn piece_on(&self, board: &Board) -> Option<Piece> {
        match self {
            Move::Drop { piece, .. } => Some(*piece),
            Move::BoardMove { from, .. } => board.piece_on(*from),
        }
    }
    */

    /// Get the source square of the move, if applicable.
    pub fn from(&self) -> Option<Square> {
        match self {
            Move::Drop { .. } => None, // Drops don't have a source square
            Move::BoardMove { from, .. } => Some(*from),
        }
    }

    /// Get the destination square of the move.
    pub fn to(&self) -> Square {
        match self {
            Move::Drop { to, .. } => *to,
            Move::BoardMove { to, .. } => *to,
        }
    }

    // Helper function to parse a square.
    fn parse_square_range(
        s: &str,
        range: core::ops::Range<usize>,
    ) -> Result<Square, MoveParseError> {
        s.get(range)
            .ok_or(MoveParseError::InvalidSquare)?
            .parse::<Square>()
            .map_err(|_| MoveParseError::InvalidSquare)
    }

    /// Parse a move string based on the notation used by Reier Grimbergen.
    ///
    /// # Grammar
    /// - `move := drop | board_move`
    /// - `drop := PIECE * square`
    /// - `board_move := piece square capture_or_hyphen square [prom]`
    /// - `piece := [+] PIECE`
    /// - `square := file rank`
    /// - `file := [1-9]`
    /// - `rank := [a-i]`
    /// - `capture_or_hyphen := [x-]`
    /// - `prom := [+=]`
    ///
    /// Note that this is not used in USI. To parse the simple USI move format,
    /// use [`Move::from_str`].
    ///
    /// # Examples
    /// ```
    /// use haitaka_types::{Move, Square, Piece};
    ///
    /// let mv = Move::parse("P*7b").unwrap();
    /// assert!(mv.is_drop());
    /// assert_eq!(mv.piece(), Some(Piece::Pawn));
    /// assert_eq!(mv.to(), Square::B7);
    /// assert!(!mv.is_promotion());
    ///
    /// let mv = Move::parse("R8bx8f").unwrap();
    /// assert!(mv.is_board_move());
    /// assert!(mv.piece().is_none());
    /// assert_eq!(mv.from(), Some(Square::B8));
    /// assert_eq!(mv.to(), Square::F8);
    /// assert!(!mv.is_promotion());
    ///
    /// let mv = Move::parse("B8hx3c+").unwrap();
    /// assert!(mv.is_board_move());
    /// assert!(mv.piece().is_none());
    /// assert_eq!(mv.from(), Some(Square::H8));
    /// assert_eq!(mv.to(), Square::C3);
    /// assert!(mv.is_promotion());
    /// ```
    pub fn parse(s: &str) -> Result<Self, MoveParseError> {
        // Check for a drop move (e.g., "P*7b")
        if let Some((piece_str, rest)) = s.split_once('*') {
            let piece = piece_str
                .parse::<Piece>()
                .map_err(|_| MoveParseError::InvalidPiece)?;
            let to = rest
                .parse::<Square>()
                .map_err(|_| MoveParseError::InvalidSquare)?;
            return Ok(Move::Drop { piece, to });
        }

        // Parse a board move (e.g., "+R8bx8f" or "B8hx3c+")
        let piece_str = if s.starts_with('+') {
            &s[0..2] // "+PIECE"
        } else {
            &s[0..1] // "PIECE"
        };
        // TODO: Review if we may still want to keep the piece
        // let piece = piece_str.parse::<Piece>().map_err(|_| MoveParseError::InvalidPiece)?;
        let n = piece_str.len();

        let from = Self::parse_square_range(s, n..n + 2)?;
        let to = Self::parse_square_range(s, n + 3..n + 5)?;

        let capture_or_hyphen = s.get(n + 2..n + 3).ok_or(MoveParseError::InvalidFormat)?;
        if capture_or_hyphen != "x" && capture_or_hyphen != "-" {
            return Err(MoveParseError::InvalidFormat);
        }

        let promotion = match s.get(n + 5..n + 6) {
            Some("+") => true,
            Some("=") => false,
            None => false,
            _ => return Err(MoveParseError::ExtraCharacters),
        };

        if s.len() > n + 6 {
            return Err(MoveParseError::ExtraCharacters);
        }

        Ok(Move::BoardMove {
            from,
            to,
            promotion,
        })
    }
}

impl FromStr for Move {
    type Err = MoveParseError;

    /// Convert a [USI](http://hgm.nubati.net/usi.html) move string into a Move.
    ///
    /// A USI move string is very similar to a UCI move string, except that drops
    /// are also supported. This representation is much simpler than the complete
    /// representation supported by [`Move::parse`].
    ///
    /// Grammar
    /// -------
    /// move := drop | board_move
    /// drop := PIECE * square
    /// board_move := square square [+]
    /// square := file rank
    /// file := [1-9]
    /// rank := [a-i]
    ///
    /// # Examples
    ///
    /// ```
    /// use haitaka_types::*;
    /// use core::str::FromStr;
    ///
    /// let mv = Move::from_str("P*7b").unwrap();
    /// assert_eq!(mv.piece(), Some(Piece::Pawn));
    /// assert_eq!(mv.from(), None);
    /// assert_eq!(mv.to(), Square::B7);
    /// assert!(!mv.is_promotion());
    ///
    /// let mv = Move::from_str("7g7f").unwrap();
    /// assert_eq!(mv.piece(), None);
    /// assert_eq!(mv.from(), Some(Square::G7));
    /// assert_eq!(mv.to(), Square::F7);
    /// assert_eq!(mv.is_promotion(), false);
    ///
    /// let mv = Move::from_str("7g7f+").unwrap();
    /// assert_eq!(mv.is_promotion(), true);
    ///
    /// assert!(Move::from_str("P*10b").is_err()); // Invalid square
    /// assert!(Move::from_str("7g").is_err()); // Too short
    /// assert!(Move::from_str("7g7f++").is_err()); // Invalid extra characters
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // drop
        if let Some((piece_str, rest)) = s.split_once('*') {
            let piece = piece_str
                .parse::<Piece>()
                .map_err(|_| MoveParseError::InvalidPiece)?;
            let to = rest
                .parse::<Square>()
                .map_err(|_| MoveParseError::InvalidSquare)?;
            return Ok(Move::Drop { piece, to });
        }

        // board move
        if s.len() < 4 {
            return Err(MoveParseError::InvalidFormat);
        }

        let from = Self::parse_square_range(s, 0..2)?;
        let to = Self::parse_square_range(s, 2..4)?;

        let promotion = match s.get(4..5) {
            // note that '=' is not in spec
            Some("+") => true,
            None => false,
            _ => return Err(MoveParseError::ExtraCharacters),
        };

        if s.len() > 5 {
            return Err(MoveParseError::ExtraCharacters);
        }

        Ok(Move::BoardMove {
            from,
            to,
            promotion,
        })
    }
}

impl core::convert::TryFrom<&str> for Move {
    type Error = MoveParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Move::from_str(s)
    }
}

impl core::fmt::Display for Move {
    /// Display a [`Move`] in [USI](http://hgm.nubati.net/usi.html) format.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Move::Drop { piece, to } => {
                write!(f, "{}*{}", piece.to_str(Color::Black), to)
            }
            Move::BoardMove {
                from,
                to,
                promotion,
            } => {
                if *promotion {
                    write!(f, "{}{}+", from, to)
                } else {
                    write!(f, "{}{}", from, to)
                }
            }
        }
    }
}
