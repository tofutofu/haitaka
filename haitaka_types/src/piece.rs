//! [`Piece`] representation
use crate::*;

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::string::String;

#[cfg(feature = "std")]
use std::string::String;

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

crate::helpers::simple_error! {
    /// The value was not a valid [`Piece`].
    pub struct PieceParseError = "The value is not a valid Piece.";
}

impl Piece {
    /// Max number of pieces for a piece type to have in hand
    pub const MAX_HAND: [u8; Self::NUM] = [
        18, // Pawn
        4,  // Lance
        4,  // Knight
        4,  // Silver
        2,  // Bishop
        2,  // Rook
        4,  // Gold
        0, 0, 0, 0, 0, 0, 0,
    ];

    // piece -> promoted piece (promoted pieces map to themselves)
    const PROMOTED: [Self; Self::NUM] = [
        Piece::Tokin,
        Piece::PLance,
        Piece::PKnight,
        Piece::PSilver,
        Piece::PBishop,
        Piece::PRook,
        Piece::Gold,
        Piece::King,
        Piece::Tokin,
        Piece::PLance,
        Piece::PKnight,
        Piece::PSilver,
        Piece::PBishop,
        Piece::PRook,
    ];

    // promoted piece -> piece (unpromoted pieces map to themselves)
    const UNPROMOTED: [Self; Self::NUM] = [
        Piece::Pawn,
        Piece::Lance,
        Piece::Knight,
        Piece::Silver,
        Piece::Bishop,
        Piece::Rook,
        Piece::Gold,
        Piece::King,
        Piece::Pawn,
        Piece::Lance,
        Piece::Knight,
        Piece::Silver,
        Piece::Bishop,
        Piece::Rook,
    ];

    /// Is this piece a promoted piece?
    #[inline(always)]
    pub const fn is_promoted(self) -> bool {
        (self as usize) >= Self::Tokin as usize
    }

    /// Is this piece a non-promoted piece?
    #[inline(always)]
    pub const fn is_unpromoted(self) -> bool {
        (self as usize) < Self::Tokin as usize
    }

    /// Can this piece ever promote?
    #[inline(always)]
    pub const fn is_promotable(self) -> bool {
        (self as usize) < Self::Gold as usize
    }

    /// Can this piece with given color promote on the given square?
    ///
    /// # Examples
    ///
    /// ```
    /// use haitaka_types::*;
    /// assert!(Piece::Pawn.can_promote(Color::Black, Square::C1));
    /// assert!(Piece::Pawn.can_promote(Color::White, Square::G1));
    /// assert!(! Piece::Pawn.can_promote(Color::Black, Square::H7));
    /// assert!(! Piece::Pawn.can_promote(Color::White, Square::C3));
    /// ```
    #[inline(always)]
    pub const fn can_promote(self, color: Color, square: Square) -> bool {
        if !self.is_promotable() {
            false
        } else {
            let rank = square.rank() as usize;
            match color {
                Color::White => rank > 5,
                Color::Black => rank < 3,
            }
        }
    }

    /// Must this piece with given color promote on the given square?
    ///
    /// # Examples
    ///
    /// ```
    /// use haitaka_types::*;
    /// assert!(Piece::Pawn.must_promote(Color::Black, Square::A3));
    /// assert!(Piece::Lance.must_promote(Color::Black, Square::A3));
    /// assert!(Piece::Knight.must_promote(Color::Black, Square::A3));
    ///
    /// assert!(Piece::Pawn.must_promote(Color::White, Square::I3));
    /// assert!(Piece::Lance.must_promote(Color::White, Square::I3));
    /// assert!(Piece::Knight.must_promote(Color::White, Square::I3));
    ///
    /// assert!(!Piece::Pawn.must_promote(Color::Black, Square::B3));
    /// assert!(!Piece::Lance.must_promote(Color::Black, Square::B3));
    /// assert!(Piece::Knight.must_promote(Color::Black, Square::B3));
    ///
    /// assert!(!Piece::Pawn.must_promote(Color::White, Square::H3));
    /// assert!(!Piece::Lance.must_promote(Color::White, Square::H3));
    /// assert!(Piece::Knight.must_promote(Color::White, Square::H3));
    /// ```
    #[inline(always)]
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
    #[inline(always)]
    pub const fn can_drop(self, color: Color, square: Square) -> bool {
        !self.must_promote(color, square)
    }

    /// Promote this piece.
    ///
    /// Never panics. If the piece cannot be promoted, it the same piece is returned.
    #[inline(always)]
    pub const fn promote(self) -> Self {
        Self::PROMOTED[self as usize]
    }

    /// Unpromote this piece.
    ///
    /// Never panics. If the piece is an unpromoted piece, the same piece is returned.
    #[inline(always)]
    pub const fn unpromote(self) -> Self {
        Self::UNPROMOTED[self as usize]
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
            _ => None,
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
            Self::PLance => "+l",
            Self::PKnight => "+n",
            Self::PSilver => "+s",
            Self::PBishop => "+b",
            Self::PRook => "+r",
        };

        if color == Color::Black {
            s.to_uppercase()
        } else {
            String::from(s)
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
