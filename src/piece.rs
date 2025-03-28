use crate::*;

//use std::str::*;
use core::fmt::*;

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
// two chars. Black/Black pieces are indicated by uppercase letters;
// White/White pieces by lowerase. Promoted pieces are indicated by
// a '+' prefix.

// TODO: Should there be a "None" or "Empty" piecetype?

crate::helpers::simple_error! {
    /// The value was not a valid [`Piece`].
    pub struct PieceParseError = "The value is not a valid Piece.";
}

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
                Color::Black => return (square as usize) >= 6 * 9,
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
            Color::White => match self {
                Self::Pawn | Self::Lance => rank == 8,
                Self::Knight => rank >= 7,
                _ => false,
            },
            Color::Black => match self {
                Self::Pawn | Self::Lance => rank == 0,
                Self::Knight => rank <= 1,
                _ => false,
            },
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
            _ => panic!("Piece can not promote."),
        }
    }

    // TODO: we need a try_prom function

    pub const fn do_promote(self, yes: bool) -> Self {
        if yes {
            self.promote()
        } else {
            self
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
            _ => self,
        }
    }

    pub fn try_from_char(c: char) -> Option<(Self, Color)> {
        match c {
            'p' => Some((Self::Pawn, Color::White)),
            'l' => Some((Self::Lance, Color::White)),
            'n' => Some((Self::Knight, Color::White)),
            's' => Some((Self::Silver, Color::White)),
            'g' => Some((Self::Gold, Color::White)),
            'r' => Some((Self::Rook, Color::White)),
            'b' => Some((Self::Bishop, Color::White)),
            'k' => Some((Self::King, Color::White)),
            'P' => Some((Self::Pawn, Color::Black)),
            'L' => Some((Self::Lance, Color::Black)),
            'N' => Some((Self::Knight, Color::Black)),
            'S' => Some((Self::Silver, Color::Black)),
            'G' => Some((Self::Gold, Color::Black)),
            'R' => Some((Self::Rook, Color::Black)),
            'B' => Some((Self::Bishop, Color::Black)),
            'K' => Some((Self::King, Color::Black)),
            _ => None
        }
    }

    pub fn try_from_str(s: &str) -> Option<(Self, Color)> {
        let mut chars = s.chars();
        let first = chars.next()?; // Get the first character
        let second = chars.next(); // Get the second character, if any

        match (first, second) {
            // Promoted pieces (e.g., "+p", "+l")
            ('+', Some(c)) => match c {
                'p' => Some((Self::Tokin, Color::White)),
                'l' => Some((Self::PLance, Color::White)),
                'n' => Some((Self::PKnight, Color::White)),
                's' => Some((Self::PSilver, Color::White)),
                'b' => Some((Self::PBishop, Color::White)),
                'r' => Some((Self::PRook, Color::White)),
                'P' => Some((Self::Tokin, Color::Black)),
                'L' => Some((Self::PLance, Color::Black)),
                'N' => Some((Self::PKnight, Color::Black)),
                'S' => Some((Self::PSilver, Color::Black)),
                'B' => Some((Self::PBishop, Color::Black)),
                'R' => Some((Self::PRook, Color::Black)),
                _ => None,
            },
            // Unpromoted pieces (e.g., "p", "l")
            (c, None) => match c {
                'p' => Some((Self::Pawn, Color::White)),
                'l' => Some((Self::Lance, Color::White)),
                'n' => Some((Self::Knight, Color::White)),
                's' => Some((Self::Silver, Color::White)),
                'g' => Some((Self::Gold, Color::White)),
                'r' => Some((Self::Rook, Color::White)),
                'b' => Some((Self::Bishop, Color::White)),
                'k' => Some((Self::King, Color::White)),
                'P' => Some((Self::Pawn, Color::Black)),
                'L' => Some((Self::Lance, Color::Black)),
                'N' => Some((Self::Knight, Color::Black)),
                'S' => Some((Self::Silver, Color::Black)),
                'G' => Some((Self::Gold, Color::Black)),
                'R' => Some((Self::Rook, Color::Black)),
                'B' => Some((Self::Bishop, Color::Black)),
                'K' => Some((Self::King, Color::Black)),
                _ => None,
            },
            // Invalid input
            _ => None,
        }
    }

    pub fn to_str(self, color: Color) -> String {
        let s: &str = match self {
            Self::Pawn => "p",
            Self::Lance => "l",
            Self::Knight => "n",
            Self::Silver => "s",
            Self::Bishop => "b",
            Self::Rook => "r",
            Self::Gold => "g",
            Self::King => "k",
            Self::Tokin => "+p",
            Self::PLance =>  "+l",
            Self::PKnight => "+n",
            Self::PSilver => "+s",
            Self::PBishop => "+b",
            Self::PRook => "+r",
        };

        if color == Color::Black {
            s.to_uppercase()
        } else {
            s.to_string()
        }
    }

}


impl core::str::FromStr for Piece {
    type Err = PieceParseError;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        Piece::try_from_str(s)
            .map(|(piece, _color)| piece) // Extract the `Piece` from the tuple
            .ok_or(PieceParseError) // Convert `Option` to `Result`
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColoredPiece {
    pub piece: Piece,
    pub color: Color,
}

impl core::str::FromStr for ColoredPiece {
    type Err = PieceParseError;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
       Piece::try_from_str(s)
           .map(|(piece, color)| ColoredPiece { piece, color }) 
           .ok_or(PieceParseError)
    }
}

impl core::fmt::Display for ColoredPiece {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.piece.to_str(self.color))
    }
}

/* 
    // Parse a ColoredPiece from a string
    let cp: ColoredPiece = "+p".parse().unwrap();
    println!("{:?}", cp); // Output: ColoredPiece { piece: Tokin, color: White }

    let cp: ColoredPiece = "P".parse().unwrap();
    println!("{:?}", cp); // Output: ColoredPiece { piece: Pawn, color: Black }

    // Invalid input
    let invalid: Result<ColoredPiece, _> = "x".parse();
    println!("{:?}", invalid); // Output: Err(PieceParseError)


    #[test]
    fn test_colored_piece_from_str() {
        // Valid inputs
        assert_eq!(
            ColoredPiece::from_str("+p"),
            Ok(ColoredPiece {
                piece: Piece::Tokin,
                color: Color::White
            })
        );

        assert_eq!(
            ColoredPiece::from_str("P"),
            Ok(ColoredPiece {
                piece: Piece::Pawn,
                color: Color::Black
            })
        );

        assert_eq!(
            ColoredPiece::from_str("+R"),
            Ok(ColoredPiece {
                piece: Piece::PRook,
                color: Color::Black
            })
        );

        assert_eq!(
            ColoredPiece::from_str("b"),
            Ok(ColoredPiece {
                piece: Piece::Bishop,
                color: Color::White
            })
        );

        // Invalid inputs
        assert_eq!(ColoredPiece::from_str("x"), Err(PieceParseError));
        assert_eq!(ColoredPiece::from_str("+x"), Err(PieceParseError));
        assert_eq!(ColoredPiece::from_str(""), Err(PieceParseError));
        assert_eq!(ColoredPiece::from_str("++p"), Err(PieceParseError));
    }


*/
