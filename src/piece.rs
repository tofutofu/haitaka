use crate::*;

//use std::str::*;
//use core::fmt::*;

crate::helpers::simple_enum! {
    /// Shogi piece types.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub enum Piece {
        /// A pawn.
        Pawn,
        /// A lance.
        Lance,
        /// A knight.
        Knight,
        /// A silver general.      
        Silver,  
        /// A bishop.
        Bishop,
        /// A rook.
        Rook,
        /// A gold general.
        Gold,
        /// A king.
        King,
        /// Tokin (promoted pawn)
        Tokin,
        /// Promoted lance.
        PLance,
        /// Promoted knight.
        PKnight,
        /// Promoted silver.
        PSilver,
        /// Promoted bishop.
        PBishop,
        /// Promoted rook.
        PRook
    }
}

// The piece representation in SFEN strings requires either one or
// two chars. Black/Sente pieces are indicated by uppercase letters;
// White/Gote pieces by lowerase. Promoted pieces are indicated by
// a '+' prefix.

// TODO: Should there be a "None" or "Empty" piecetype?

crate::helpers::simple_error! {
    /// The value was not a valid [`Piece`].
    pub struct PieceParseError = "The value is not a valid Piece.";
}

/* 
impl core::str::FromStr for Piece {
    type Err = PieceParseError;

    /// Convert a string slice into a Piece.
    /// 
    /// This function ignores the color of the piece, so its perhaps not terribly useful.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use core::convert::TryInto;

        let mut chars = s.chars();
        let first = chars.next().ok_or(PieceParseError)?;
        let sec = chars.next();

        match (first, sec) {
            ('+', Some(c)) => {
                let mut cc = c.to_lowercase();                
                match cc {
                    'p' => Ok(Piece::Tokin),
                    'l' => Ok(Piece::PLance),
                    'n' => Ok(Piece::PKnight),
                    's' => Ok(Piece::PSilver),
                    'b' => Ok(Piece::PBishop),
                    'r' => Ok(Piece::PRook),
                    _ => Err(PieceParseError),
                }
            }
            (c, None) => {
                let mut cc = c.to_lowercase();
                match cc {
                    'p' => Ok(Piece::Pawn),
                    'l' => Ok(Piece::Lance),
                    'n' => Ok(Piece::Knight),
                    's' => Ok(Piece::Silver),
                    'b' => Ok(Piece::Bishop),
                    'r' => Ok(Piece::Rook),
                    'g' => Ok(Piece::Gold),
                    'k' => Ok(Piece::King),
                    _ => Err(PieceParseError),
                }
            }
            _ => Err(PieceParseError),
        }
    }
}

impl core::fmt::Display for Piece {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        let s = match self {
            Piece::Pawn => "p",
            Piece::Lance => "l",
            Piece::Knight => "n",
            Piece::Silver => "s",
            Piece::Bishop => "b",
            Piece::Rook => "r",
            Piece::Gold => "g",
            Piece::King => "k",
            Piece::Tokin => "+p",
            Piece::PLance => "+l",
            Piece::PKnight => "+n",
            Piece::PSilver => "+s",
            Piece::PBishop => "+b",
            Piece::PRook => "+r",
        };
        write!(f, "{}", s)
    }
}


impl std::str::FromStr for (Color, Piece) {
    type Err = PieceParseError;

    /// Convert a string slice into a tuple (color, piecetype).
    ///
    /// # Examples
    ///
    /// ```
    /// use sparrow::{Color, Piece};
    /// use std::str::FromStr;
    ///
    /// let piece: (Color, Piece) = (Color, Piece)::from_str("p").unwrap();
    /// assert_eq!(piece, (Color::White, Piece::Pawn));
    ///
    /// let piece: (Color, Piece) = (Color, Piece)::from_str("P").unwrap();
    /// assert_eq!(piece, (Color::Black, Piece::Pawn));
    ///
    /// let piece: (Color, Piece) = (Color, Piece)::from_str("+p").unwrap();
    /// assert_eq!(piece, (Color::White, Piece::Tokin));
    ///
    /// let piece: (Color, Piece) = (Color, Piece)::from_str("+P").unwrap();
    /// assert_eq!(piece, (Color::Black, Piece::Tokin));
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 1 {
            let c = s.chars().next().unwrap();
            let color = if c.is_uppercase() { Color::Black } else { Color::White };
            let piece_char = c.to_lowercase().next().unwrap();
            let piece = match piece_char {
                'p' => Ok(Piece::Pawn),
                'l' => Ok(Piece::Lance),
                'n' => Ok(Piece::Knight),
                's' => Ok(Piece::Silver),
                'b' => Ok(Piece::Bishop),
                'r' => Ok(Piece::Rook),
                'g' => Ok(Piece::Gold),
                'k' => Ok(Piece::King),
                _ => Err(PieceParseError),
            }?;
            Ok((color, piece))
        } else if s.len() == 2 && s.starts_with('+') {
            let c = s.chars().nth(1).unwrap();
            let color = if c.is_uppercase() { Color::Black } else { Color::White };
            let piece_char = c.to_lowercase().next().unwrap();
            let piece = match piece_char {
                'p' => Ok(Piece::Tokin),
                'l' => Ok(Piece::PLance),
                'n' => Ok(Piece::PKnight),
                's' => Ok(Piece::PSilver),
                'b' => Ok(Piece::PBishop),
                'r' => Ok(Piece::PRook),
                _ => Err(PieceParseError),
            }?;
            Ok((color, piece))
        } else {
            Err(PieceParseError)
        }
    }
}


impl fmt::Display for (Color, Piece) {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let piece_str = format!("{}", self.1);
        let formatted_str = match self.0 {
            Color::White => piece_str,
            Color::Black => piece_str.to_uppercase(),
        };
        write!(f, "{}", formatted_str)
    }
}

*/

impl Piece {

    /// Is this piece a promoted piece?
    pub const fn is_promoted(self) -> bool {
        return (self as usize) > Self::King as usize;
    }

    /// Is this piece a non-promoted piece?
    pub const fn is_unpromoted(self) -> bool {
        return (self as usize) < Self::Tokin as usize;
    }

    /// Can this piece ever promote?
    pub const fn is_promotable(self) -> bool {
        return (self as usize) < Self::Gold as usize;
    }

    /// Can this piece with given color promote on the given square?
    pub const fn can_promote(self, color: Color, square: Square) -> bool {
        if self.is_promotable() {
            match color {
                Color::White => return (square as usize) < 3 * 9,
                Color::Black => return (square as usize) >= 6 * 9
            }
        }
        false
    }

    /// Must this piece with given color promote on the given square?
    pub const fn must_promote(self, color: Color, square: Square) -> bool {
        let rank = square.rank() as usize;
        if 1 < rank && rank < 7 {
            return false;
        }
        match color {
            Color::White => {
                match self {
                    Self::Pawn | Self::Lance => rank == 8,
                    Self::Knight => rank >= 7,
                    _ => false
                }
            },
            Color::Black => {
                match self {
                    Self::Pawn | Self::Lance => rank == 0,
                    Self::Knight => rank <= 1,
                    _ => false
                }
            }
        }
    }

    /// Can this piece with given color be dropped on the given square?
    /// If the piece must promote on the square, it can not be dropped there.
    /// 
    /// This function does not perform any occupancy check.
    /// It assumes the given square is empty.
    /// 
    pub const fn can_drop(self, color: Color, square: Square) -> bool {
        !self.must_promote(color, square)
    }

    // TODO: See if it makes things a lot faster to use static arrays
    // to get the promoted or unpromoted pieces.

    /// Promote this piece.
    /// # Panics
    /// This function panics if the piece can not be promoted.
    pub const fn promote(self) -> Self {
        match self {
            Self::Pawn => Self::Tokin,
            Self::Lance => Self::PLance,
            Self::Knight => Self::PKnight,
            Self::Silver => Self::PSilver,
            Self::Bishop => Self::PBishop,
            Self::Rook => Self::PRook,
            _ => panic!("Piece can not promote.")
        }
    }

    /// Unpromote this piece.
    /// 
    /// Note: This function does not panic. If called on
    /// an unpromoted piece, that piece will simply be returned.
    pub const fn unpromote(self) -> Self {
        match self {
            Self::Tokin => Self::Pawn,
            Self::PSilver => Self::Silver,
            Self::PKnight => Self::Knight,
            Self::PBishop => Self::Bishop,
            Self::PRook => Self::Rook,
            _ => self
        }
    }

}

