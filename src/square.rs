//! This module defines the Square enum to represent squares on a Shogi board.
//! 
//! By Japanese convention squares are written as {file}{rank}. For instance, the topmost
//! rightmost square in board diagrams is written as "1a", "11", or "1一"; the center
//! square is written as "5e", "55", or "5五". Internally we represent square "1a" as
//! Square::A1, and square "5e" as Square::E5.
//! 
//! Squares are ordered internally in file-major order: A1, B1, C1, ... I8, I9. This
//! means that the squares on the rightmost file (File::One) corresponds to the LSB
//! (least significant) bits of the bitboards.
//!    
use core::convert::TryInto;
use core::str::FromStr;

use crate::*;

macro_rules! define_square_with_docs {
    ($($square:ident),*) => {
        crate::helpers::simple_enum! {
            /// A square on a Shogi board.
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
            pub enum Square {
                $(
                    #[doc = concat!("The ", stringify!($square), " square.")]
                    $square
                ),*
            }
        }
    }
}

define_square_with_docs! {    
    A1, B1, C1, D1, E1, F1, G1, H1, I1,
    A2, B2, C2, D2, E2, F2, G2, H2, I2,
    A3, B3, C3, D3, E3, F3, G3, H3, I3,
    A4, B4, C4, D4, E4, F4, G4, H4, I4,
    A5, B5, C5, D5, E5, F5, G5, H5, I5,
    A6, B6, C6, D6, E6, F6, G6, H6, I6,
    A7, B7, C7, D7, E7, F7, G7, H7, I7,
    A8, B8, C8, D8, E8, F8, G8, H8, I8,
    A9, B9, C9, D9, E9, F9, G9, H9, I9
}


crate::helpers::simple_error! {
    /// The value was not a valid [`Square`].
    pub struct SquareParseError = "The value was not a valid Square.";
}

impl FromStr for Square {
    type Err = SquareParseError;

    // "1a" => File::One, Rank::A => Square::A1
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let file = chars.next()
            .and_then(|c| c.try_into().ok())
            .ok_or(SquareParseError)?;
        let rank = chars.next()
            .and_then(|c| c.try_into().ok())
            .ok_or(SquareParseError)?;
        if chars.next().is_some() {
            return Err(SquareParseError);
        }
        Ok(Square::new(file, rank))
    }
}

impl core::fmt::Display for Square {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{}{}", self.file(), self.rank())
    }
}

impl Square {

    /// Make a square from a file and a rank.
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// assert_eq!(Square::new(File::One, Rank::A), Square::A1);
    /// assert_eq!(Square::new(File::Two, Rank::B), Square::B2);
    /// ```
    #[inline(always)]
    pub const fn new(file: File, rank: Rank) -> Self {
        Self::index_const((file as usize) * 9 + (rank as usize))
    }

    /// Get the file of this square.
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// assert_eq!(Square::A1.file(), File::One);
    /// assert_eq!(Square::B2.file(), File::Two);
    /// ```
    #[inline(always)]
    pub const fn file(self) -> File {
        File::index_const(self as usize / 9)
    }

    /// Get the rank of this square.
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// assert_eq!(Square::A1.rank(), Rank::A);
    /// assert_eq!(Square::B2.rank(), Rank::B);
    /// ```
    #[inline(always)]
    pub const fn rank(self) -> Rank {
        Rank::index_const(self as usize % 9)
    }

    /// Get a bitboard with this square set.
    /// ```
    /// # use sparrow::*;
    /// assert_eq!(Square::G8.bitboard(), bitboard! {
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . X . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    /// });
    /// ```
    #[inline(always)]
    pub const fn bitboard(self) -> BitBoard {
        BitBoard(1 << self as usize)
    }

    /// Add a file and rank offset to the given square.
    /// 
    /// Since square A1 is the topmost-rightmost square,
    /// positive offsets correspond to a down- and leftwards
    /// direction. Note that A1 means file 1 and rank A.
    /// 
    /// # Panics
    /// Panic if the offset would put the square out of bounds.
    /// See [`Square::try_offset`] for a non-panicking variant.
    /// 
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// assert_eq!(Square::A1.offset(2, 1), Square::B3);
    /// assert_eq!(Square::B3.offset(-2, -1), Square::A1);    
    /// ```
    pub const fn offset(self, file_offset: i8, rank_offset: i8) -> Square {
        if let Some(sq) = self.try_offset(file_offset, rank_offset) {
            sq
        } else {
            panic!("Offset puts square out of bounds.")
        }
    }

    /// Non-panicking version of [`Square::offset`].
    /// # Errors
    /// See [`Square::offset`]'s panics.
    #[inline(always)]
    pub const fn try_offset(self, file_offset: i8, rank_offset: i8) -> Option<Square> {
        let file_index = self.file() as i8 + file_offset;
        let rank_index = self.rank() as i8 + rank_offset;

        if file_index < 0 || file_index >= 8 || rank_index < 0 || rank_index >= 8 {
            return None;
        }
        Some(Square::new(
            File::index_const(file_index as usize),
            Rank::index_const(rank_index as usize)
        ))
    }

    /// Flip the file of this square.
    /// 
    /// Mirrors square in the central File::Five.
    /// 
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// assert_eq!(Square::A1.flip_file(), Square::A9);
    /// ```
    #[inline(always)]
    pub const fn flip_file(self) -> Self {
        Self::new(self.file().flip(), self.rank())
    }

    /// Flip the rank of this square.
    /// 
    /// Mirrors square in the central Rank::E.
    /// 
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// assert_eq!(Square::A1.flip_rank(), Square::I1);
    /// ```
    #[inline(always)]
    pub const fn flip_rank(self) -> Self {
        Self::new(self.file(), self.rank().flip())
    }

    /// Flip both rank and file of this square.
    /// 
    /// This rotates the square around the center square E5.
    /// 
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// assert_eq!(Square::A1.flip(), Square::I9);
    /// assert_eq!(Square::E5.flip(), Square::E5);
    /// ```
    #[inline(always)]
    pub const fn flip(self) -> Self {
        Self::new(self.file().flip(), self.rank().flip())
    }


    /// Get a square relative to some color.
    /// 
    /// This effectively _rotates_ the board if viewed from Gote's/White's
    /// perspective. It flips both the rank and the file of the square.
    /// 
    /// Note that the initial Shogi position has rotational symmetry.
    /// This differs from the initial position in International Chess which has
    /// mirror symmetry (flipping the ranks).
    ///   
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// assert_eq!(Square::A1.relative_to(Color::White), Square::I9);
    /// assert_eq!(Square::E5.relative_to(Color::White), Square::E5);
    /// assert_eq!(Square::A1.relative_to(Color::Black), Square::A1);
    /// ```
    #[inline(always)]
    pub const fn relative_to(self, color: Color) -> Self {
        if let Color::Black = color {
            self
        } else {
            Self::new(
                self.file().flip(), 
                self.rank().flip()
            )
        }
    }
}
