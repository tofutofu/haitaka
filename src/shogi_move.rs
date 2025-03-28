use core::str::FromStr;

use crate::*;

// TODO: Check against common formats (SFEN, KIF)

/// A Shogi move.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move {
    /// The square to move the piece from.
    pub from: Square,
    /// The square to move the piece to.
    pub to: Square,
    /// Flag to indicate if piece promotes or not.
    pub promotion: bool
}

crate::helpers::simple_error! {
    /// The value was not a valid [`Move`].
    pub struct MoveParseError = "The value was not a valid Move.";
}

impl FromStr for Move {
    type Err = MoveParseError;

    /// Convert a string into a Move.
    /// 
    /// # Examples
    ///
    /// ```
    /// use sparrow::{Move, Square};
    /// use core::str::FromStr;
    ///
    /// let mv = Move::from_str("7g7f").unwrap();
    /// assert_eq!(mv.from, Square::from_str("7g").unwrap());
    /// assert_eq!(mv.to, Square::from_str("7f").unwrap());
    /// assert_eq!(mv.from, Square::G7);
    /// assert_eq!(mv.to, Square::F7);
    /// assert_eq!(mv.promotion, false);
    ///
    /// let mv = Move::from_str("7g7f+").unwrap();
    /// assert_eq!(mv.from, Square::from_str("7g").unwrap());
    /// assert_eq!(mv.to, Square::from_str("7f").unwrap());
    /// assert_eq!(mv.from, Square::G7);
    /// assert_eq!(mv.to, Square::F7);
    /// assert_eq!(mv.promotion, true);
    /// ```

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse(s: &str) -> Option<Move> {
            Some(Move {
                from: s.get(0..2)?.parse().ok()?,
                to: s.get(2..4)?.parse().ok()?,
                promotion: s.get(4..5) == Some("+")
            })
        }
        parse(s).ok_or(MoveParseError)
    }
}

impl core::fmt::Display for Move {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.promotion {
            write!(f, "{}{}+", self.from, self.to)?;
        } else {
            write!(f, "{}{}", self.from, self.to)?;
        }
        Ok(())
    }
}
