//! The [`Square`] enum represent squares on a Shogi board
//!
//! By Japanese convention squares are written as {file}{rank}. For instance, the topmost
//! rightmost square in board diagrams is written either as "1a", "11", or "1一";
//! the center square is written as "5e", "55", or "5五". Internally we represent square
//! "1a" as Square::A1, and square "5e" as Square::E5. So, other than in Internation Chess,
//! ranks (rows) are indicated by letters and files (columns) by numerals.
//!
//! Squares are ordered internally in file-major order: A1, B1, C1, ... I8, I9. This
//! means that the squares on the rightmost file (File::One) correspond to the LSB bits
//! of the bitboards. The main reason for choosing this internal layout is that it
//! makes move generation of Lance moves easier to implement and faster (since Lances
//! slide along files).
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

// Defining the squares in file-major order.
// Note: Changing this order will break BitBoard::has (and other functions).

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
        let file = chars
            .next()
            .and_then(|c| c.try_into().ok())
            .ok_or(SquareParseError)?;
        let rank = chars
            .next()
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

// Directions  Diagrams     Square indices
// NW N NE     A9 ... A1    72 ...  0
//  W . E         ...          ...
// SW S SE     I9 ... I1    80 ...  9

// Diagonal mask for forward ("up") slashing diagonals (/).
const NEG_MASK: u128 = 0x100401004010040100401;

// Diagonal mask for backward ("down") slashing diagonals (\).
const POS_MASK: u128 = 0x1010101010101010100;

// BitBoards for the two main diagonals.
const NEGD: BitBoard = BitBoard::new(NEG_MASK);
const POSD: BitBoard = BitBoard::new(POS_MASK);

/// Down-slanting diagonals.
///
/// A square (file, rank) is on down-slanting diagonal `POS_DIA[file + rank]`.
/// The down-slanting diagonals are indexed as
/// ```text
///    8  7  6  5  4  3  2  1  0
///    9  8  7  6  5  4  3  2  1
///   10  9  8  7  6  5  4  3  2
///   11 10  9  8  7  6  5  4  3
///   12 11 10  9  8  7  6  5  4
///   13 12 11 10  9  8  7  6  5
///   13 12 11 10  9  8  7  6  5
///   14 13 12 11 10  9  8  7  6
///   15 14 13 12 11 10  9  8  7
///   16 15 14 13 12 11 10  9  8
/// ```
///
/// # Examples
/// ```
/// use haitaka::*;
/// assert_eq!(POS_DIA[8], bitboard! {
///     X . . . . . . . .
///     . X . . . . . . .
///     . . X . . . . . .
///     . . . X . . . . .
///     . . . . X . . . .
///     . . . . . X . . .
///     . . . . . . X . .
///     . . . . . . . X .
///     . . . . . . . . X
/// });
/// assert_eq!(POS_DIA[2], bitboard! {
///     . . . . . . X . .
///     . . . . . . . X .
///     . . . . . . . . X
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
/// });
/// assert_eq!(POS_DIA[16], bitboard! {
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     X . . . . . . . .
/// });
/// ```
pub const POS_DIA: [BitBoard; 17] = [
    POSD.shr(8),
    POSD.shr(7),
    POSD.shr(6),
    POSD.shr(5),
    POSD.shr(4),
    POSD.shr(3),
    POSD.shr(2),
    POSD.shr(1),
    POSD,
    POSD.shl(1),
    POSD.shl(2),
    POSD.shl(3),
    POSD.shl(4),
    POSD.shl(5),
    POSD.shl(6),
    POSD.shl(7),
    POSD.shl(8),
];

/// Up-slanting diagonals.
///
/// A square (file, rank) is on up-slanting diagonal NEG_DIA[8 + rank - file].
/// The upslanting diagonals are indexed as
/// ```text
///     0  1  2  3  4  5  6  7  8
///     1  2  3  4  5  6  7  8  9
///     2  3  4  5  6  7  8  9 10
///     3  4  5  6  7  8  9 10 11
///     4  5  6  7  8  9 10 11 12
///     5  6  7  8  9 10 11 12 13
///     6  7  8  9 10 11 12 13 14
///     7  8  9 10 11 12 13 14 15
///     8  9 10 11 12 13 14 15 16
/// ```
///
/// # Examples
/// ```
/// use haitaka::*;
/// assert_eq!(NEG_DIA[8], bitboard! {
///     . . . . . . . . X
///     . . . . . . . X .
///     . . . . . . X . .
///     . . . . . X . . .
///     . . . . X . . . .
///     . . . X . . . . .
///     . . X . . . . . .
///     . X . . . . . . .
///     X . . . . . . . .
/// });
/// assert_eq!(NEG_DIA[14], bitboard! {
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . X
///     . . . . . . . X .
///     . . . . . . X . .
/// });
/// assert_eq!(NEG_DIA[0], bitboard! {
///     X . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .     
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
///     . . . . . . . . .
/// });
/// ```
pub const NEG_DIA: [BitBoard; 17] = [
    NEGD.shr(8),
    NEGD.shr(7),
    NEGD.shr(6),
    NEGD.shr(5),
    NEGD.shr(4),
    NEGD.shr(3),
    NEGD.shr(2),
    NEGD.shr(1),
    NEGD,
    NEGD.shl(1),
    NEGD.shl(2),
    NEGD.shl(3),
    NEGD.shr(4),
    NEGD.shl(5),
    NEGD.shl(6),
    NEGD.shl(7),
    NEGD.shl(8),
];

impl Square {
    /// Make a square from a file and a rank.
    /// # Examples
    /// ```
    /// # use haitaka::*;
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
    /// # use haitaka::*;
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
    /// # use haitaka::*;
    /// assert_eq!(Square::A1.rank(), Rank::A);
    /// assert_eq!(Square::B2.rank(), Rank::B);
    /// ```
    #[inline(always)]
    pub const fn rank(self) -> Rank {
        Rank::index_const(self as usize % 9)
    }

    /// Get a bitboard with this square set.
    /// ```
    /// # use haitaka::*;
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

    /// Get the bitboard with the "up" (forward-slanting "/") diagonal for this square.
    ///
    /// # Examples
    /// ```
    /// use haitaka::*;
    /// assert_eq!(Square::E5.up_diagonal(), bitboard! {
    ///     . . . . . . . . X
    ///     . . . . . . . X .
    ///     . . . . . . X . .
    ///     . . . . . X . . .
    ///     . . . . X . . . .
    ///     . . . X . . . . .
    ///     . . X . . . . . .
    ///     . X . . . . . . .
    ///     X . . . . . . . .
    /// });
    /// assert_eq!(Square::A1.up_diagonal(), Square::I9.up_diagonal());
    /// assert_eq!(Square::A9.up_diagonal(), Square::A9.bitboard());
    /// assert_eq!(Square::I1.up_diagonal(), Square::I1.bitboard());
    /// ```
    #[inline(always)]
    pub const fn up_diagonal(self) -> BitBoard {
        let rank = self as usize % 9;
        let file = self as usize / 9;
        NEG_DIA[8 + rank - file]
    }

    /// Get the bitboard with the "down" (backwards-slanting "\") diagonal for this square.
    ///
    /// # Examples
    /// ```
    /// use haitaka::*;
    /// assert_eq!(Square::E5.down_diagonal(), bitboard! {
    ///     X . . . . . . . .
    ///     . X . . . . . . .
    ///     . . X . . . . . .
    ///     . . . X . . . . .
    ///     . . . . X . . . .
    ///     . . . . . X . . .
    ///     . . . . . . X . .
    ///     . . . . . . . X .
    ///     . . . . . . . . X
    /// });
    /// assert_eq!(Square::A9.down_diagonal(), Square::I1.down_diagonal());
    /// assert_eq!(Square::A1.down_diagonal(), Square::A1.bitboard());
    /// assert_eq!(Square::I9.down_diagonal(), Square::I9.bitboard());
    /// ```
    #[inline(always)]
    pub const fn down_diagonal(self) -> BitBoard {
        let rank = self as usize % 9;
        let file = self as usize / 9;
        POS_DIA[file + rank]
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
    /// # use haitaka::*;
    /// assert_eq!(Square::A1.offset(2, 1), Square::B3);
    /// assert_eq!(Square::B3.offset(-2, -1), Square::A1);  
    /// assert_eq!(Square::H1.offset(0, 1), Square::I1);
    /// assert_eq!(Square::H9.offset(0, 1), Square::I9);
    /// ```
    pub const fn offset(self, file_offset: i8, rank_offset: i8) -> Square {
        if let Some(sq) = self.try_offset(file_offset, rank_offset) {
            sq
        } else {
            panic!("Offset puts square out of bounds");
        }
    }

    /// Non-panicking version of [`Square::offset`].
    ///
    /// # Errors
    /// See [`Square::offset`]'s panics.
    ///
    /// # Examples
    /// ```
    /// use haitaka::*;
    /// assert_eq!(Square::A1.try_offset(1, 1), Some(Square::B2));
    /// assert_eq!(Square::E5.try_offset(-1, -1), Some(Square::D4));
    /// assert_eq!(Square::H9.try_offset(0, -1), Some(Square::G9));
    /// assert_eq!(Square::C3.try_offset(2, 0), Some(Square::C5));
    /// assert_eq!(Square::A1.try_offset(-1, 0), None); // File out of bounds
    /// assert_eq!(Square::A1.try_offset(0, -1), None); // Rank out of bounds
    /// assert_eq!(Square::I9.try_offset(1, 0), None); // File out of bounds
    /// assert_eq!(Square::I9.try_offset(0, 1), None); // Rank out of bounds
    /// ```
    #[inline(always)]
    pub const fn try_offset(self, file_offset: i8, rank_offset: i8) -> Option<Square> {
        let file_index = self.file() as i8 + file_offset;
        let rank_index = self.rank() as i8 + rank_offset;

        if file_index < 0 || file_index >= 9 || rank_index < 0 || rank_index >= 9 {
            return None;
        }
        Some(Square::new(
            File::index_const(file_index as usize),
            Rank::index_const(rank_index as usize),
        ))
    }

    /// Flip the file of this square.
    ///
    /// Mirrors square in the central File::Five.
    ///
    /// # Examples
    /// ```
    /// # use haitaka::*;
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
    /// # use haitaka::*;
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
    /// # use haitaka::*;
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
    /// # use haitaka::*;
    /// assert_eq!(Square::A1.relative_to(Color::White), Square::I9);
    /// assert_eq!(Square::E5.relative_to(Color::White), Square::E5);
    /// assert_eq!(Square::A1.relative_to(Color::Black), Square::A1);
    /// ```
    #[inline(always)]
    pub const fn relative_to(self, color: Color) -> Self {
        if let Color::Black = color {
            self
        } else {
            Self::new(self.file().flip(), self.rank().flip())
        }
    }
}
