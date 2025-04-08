//! The [`File`] enum represents the files (columns) on a Shogi board
//!
//! Shogi files are indicated by arabic numerals. In diagrams file 1 (File::One) is the east-most column, at the westhand-side of the Gote
//! (White) player. File 9 (File::Nine) is the west-most column, at the westhand side of the Sente (Black) player.
//!
//!
use crate::*;

crate::helpers::simple_enum! {
    /// A file (column) on a shogi board.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub enum File {
        /// File One
        One,
        /// File Two
        Two,
        /// File Three
        Three,
        /// File Four
        Four,
        /// File Five
        Five,
        /// File Six
        Six,
        /// File Seven
        Seven,
        /// File Eight
        Eight,
        /// File Nine
        Nine
    }
}

crate::helpers::enum_char_conv! {
    File, FileParseError {
        One = '1',
        Two = '2',
        Three = '3',
        Four = '4',
        Five = '5',
        Six = '6',
        Seven = '7',
        Eight = '8',
        Nine ='9'
    }
}

impl File {
    /// Flip the file horizontally around the central file File::Five.
    ///
    /// This mirrors the file in the central file. It maps File::One to
    /// File::Nine, and vice-versa, and similar for all other files.
    ///
    /// # Examples
    ///
    /// ```
    /// # use haitaka::*;
    /// assert_eq!(File::Five.flip(), File::Five);
    /// assert_eq!(File::One.flip(), File::Nine);
    /// ```
    #[inline(always)]
    pub const fn flip(self) -> Self {
        Self::index_const(Self::Nine as usize - self as usize)
    }

    /// Get a bitboard with all squares on this file set.
    ///
    /// File 1 is the east-most file board diagrams.
    ///
    /// # Examples
    /// ```
    /// # use haitaka::*;
    /// assert_eq!(File::Eight.bitboard(), bitboard! {
    ///     . X . . . . . . .
    ///     . X . . . . . . .
    ///     . X . . . . . . .
    ///     . X . . . . . . .
    ///     . X . . . . . . .
    ///     . X . . . . . . .
    ///     . X . . . . . . .
    ///     . X . . . . . . .
    ///     . X . . . . . . .
    /// });
    /// ```
    #[inline(always)]
    pub const fn bitboard(self) -> BitBoard {
        BitBoard(0x1FF << (9 * (self as usize)))
    }

    /// Get a bitboard with all squares to the West of this file set.
    ///
    /// # Examples
    ///
    /// ```
    /// use haitaka::*;
    /// assert_eq!(File::Nine.west(), BitBoard::EMPTY);
    /// assert_eq!(File::Eight.west(), File::Nine.bitboard());
    /// assert_eq!(File::Two.west(), bitboard!{
    ///     X X X X X X X . .
    ///     X X X X X X X . .
    ///     X X X X X X X . .
    ///     X X X X X X X . .
    ///     X X X X X X X . .
    ///     X X X X X X X . .
    ///     X X X X X X X . .
    ///     X X X X X X X . .
    ///     X X X X X X X . .
    /// });
    /// ```
    #[inline(always)]
    pub const fn west(self) -> BitBoard {
        BitBoard::new(BitBoard::FULL.0 << (9 * (self as usize + 1)))
    }

    /// Get a bitboard with all squares to the East of this file set.
    ///
    /// # Examples
    ///
    /// ```
    /// use haitaka::*;
    /// assert_eq!(File::One.east(), BitBoard::EMPTY);
    /// assert_eq!(File::Two.east(), File::One.bitboard());
    /// assert_eq!(File::Seven.east(), bitboard!{
    ///     . . . X X X X X X
    ///     . . . X X X X X X
    ///     . . . X X X X X X
    ///     . . . X X X X X X
    ///     . . . X X X X X X
    ///     . . . X X X X X X
    ///     . . . X X X X X X
    ///     . . . X X X X X X
    ///     . . . X X X X X X
    /// });
    /// ```
    #[inline(always)]
    pub const fn east(self) -> BitBoard {
        BitBoard::new(BitBoard::FULL.0 >> (9 * (9 - self as usize)))
    }
}
