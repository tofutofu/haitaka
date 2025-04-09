//! A [`BitBoard`] implementation
use crate::{File, Rank, Square};
use core::ops::*;

/// A [bitboard](https://www.chessprogramming.org/Bitboards).
/// A bitboard is an ordered set of squares. The set contains a square if bit `1 << square as usize` is set.
///
/// Logical operators are overloaded to work as set operations.
///
/// # Examples
///  
/// ```
/// # use haitaka_types::*;
/// let a1 = Square::A1.bitboard();
/// let b1 = Square::B1.bitboard();
/// let c1 = Square::C1.bitboard();
/// let x = a1 | b1;
/// let y = a1 | c1;
/// // Union
/// assert_eq!(x | y, a1 | b1 | c1);
/// // Intersection
/// assert_eq!(x & y, a1);
/// // Symmetric difference
/// assert_eq!(x ^ y, b1 | c1);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct BitBoard(
    /// The backing [`u128`].
    pub u128,
);

macro_rules! impl_math_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait for BitBoard {
            type Output = Self;

            #[inline(always)]
            fn $fn(self, rhs: Self) -> Self::Output {
                self.$fn(rhs)
            }
        }
    )*};
}
impl_math_ops! {
    BitAnd, bitand;
    BitOr, bitor;
    BitXor, bitxor;
}

macro_rules! impl_math_assign_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait for BitBoard {
            #[inline(always)]
            fn $fn(&mut self, rhs: Self) {
                $trait::$fn(&mut self.0, rhs.0)
            }
        }
    )*};
}
impl_math_assign_ops! {
    BitAndAssign, bitand_assign;
    BitOrAssign, bitor_assign;
    BitXorAssign, bitxor_assign;
}

impl Sub for BitBoard {
    type Output = Self;

    /// Calculate the set difference (a - b).
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        self & !rhs
    }
}

impl SubAssign for BitBoard {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Not for BitBoard {
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self::Output {
        self.not()
    }
}

impl Shl<usize> for BitBoard {
    type Output = BitBoard;

    #[inline(always)]
    fn shl(self, rhs: usize) -> BitBoard {
        self.shl(rhs)
    }
}

impl Shr<usize> for BitBoard {
    type Output = BitBoard;

    #[inline(always)]
    fn shr(self, rhs: usize) -> BitBoard {
        self.shr(rhs)
    }
}

// Convert File, Rank or Square to BitBoard
macro_rules! impl_convert {
    ($($type:ty),*) => {$(
        impl From<$type> for BitBoard {
            fn from(value: $type) -> Self {
                value.bitboard()
            }
        }
    )*};
}
impl_convert!(File, Rank, Square);

//
// Logical operators and shifts implemented as `const` functions.
//
impl BitBoard {
    /// Invert all bits of this bitboard.
    #[inline(always)]
    pub const fn not(self) -> Self {
        Self(!self.0 & BitBoard::BOARD_MASK)
    }

    /// Return the intersection of two bitboards.
    #[inline(always)]
    pub const fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }

    /// Return the union of two bitboards.
    #[inline(always)]
    pub const fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }

    /// Return the symmetric set difference (xor) of two bitboards.
    #[inline(always)]
    pub const fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }

    /// Decrement. Substracts 1 from the internal u128.
    ///
    /// # Examples
    ///
    /// ```
    /// # use haitaka_types::*;
    /// assert_eq!(BitBoard::EMPTY.dec(), BitBoard::FULL);
    /// assert_eq!(Square::A1.bitboard().dec(), BitBoard::EMPTY);
    /// assert_eq!(Square::A2.bitboard().dec(), BitBoard(0x1FF));
    /// ```
    #[inline(always)]
    pub const fn dec(self) -> Self {
        Self::new(self.0.wrapping_sub(1))
    }

    /// Shift left along files ("down" the board towards rank I).
    ///
    /// # Example
    /// ```
    /// # use haitaka_types::*;
    /// let bb1 = bitboard! {
    ///     . . . . . . X X X
    ///     . . . . . . X . X
    ///     . . . . . . X X X
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     X X X . . . . . .
    ///     X . X . . . . . .
    ///     X X X . . . . . .
    /// };
    /// let bb2 = bitboard! {
    ///     . . . . . . . . .
    ///     . . . . . . X X X
    ///     . . . . . . X . X
    ///     . . . . . . X X X
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     X X X . . . . . .
    ///     X . X . . . . . .
    /// };
    /// assert_eq!(bb1.shl(1), bb2);
    /// ```
    #[inline(always)]
    pub const fn shl(self, rhs: usize) -> Self {
        if rhs == 0 {
            self
        } else if rhs >= 9 {
            BitBoard::EMPTY
        } else {
            BitBoard::new((self.0 << rhs) & Rank::SOUTH[rhs - 1].0)
        }
    }

    /// Shift right along files ("up" the board towards rank A).
    ///
    /// # Example
    ///
    /// ```
    /// # use haitaka_types::*;
    /// let bb1 = bitboard! {
    ///     . . . . . . X X X
    ///     . . . . . . X . X
    ///     . . . . . . X X X
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     X X X . . . . . .
    ///     X . X . . . . . .
    ///     X X X . . . . . .
    /// };
    /// let bb2 = bitboard! {
    ///     . . . . . . X . X
    ///     . . . . . . X X X
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     X X X . . . . . .
    ///     X . X . . . . . .
    ///     X X X . . . . . .
    ///     . . . . . . . . .
    /// };
    /// assert_eq!(bb1.shr(1), bb2);
    /// assert_eq!(bb1.shr(9), BitBoard::EMPTY);
    /// ```
    #[inline(always)]
    pub const fn shr(self, rhs: usize) -> Self {
        if rhs == 0 {
            self
        } else if rhs >= 9 {
            BitBoard::EMPTY
        } else {
            BitBoard::new((self.0 >> rhs) & Rank::NORTH[9 - rhs].0)
        }
    }

    /// Shift the bit set pattern North.
    ///
    /// The `shift_*` functions assume the usual rendering of the Shogi board
    /// where square A1 is the top-most, right-most square ("north east") and
    /// I9 the lowest, left-most one ("south west").
    #[inline(always)]
    pub const fn shift_north(self, dy: usize) -> Self {
        self.shr(dy)
    }

    /// Shift the bit set pattern South.
    #[inline(always)]
    pub const fn shift_south(self, dy: usize) -> Self {
        self.shl(dy)
    }

    /// Shift the bit set pattern vertically.
    ///
    /// This shifts the bit set up if `dy < 0`, otherwise down.
    ///
    /// # Panics
    /// This will panic if the shift amount is out of range (abs(dy) > 9).
    ///
    #[inline(always)]
    pub const fn shift_along_file(self, dy: i32) -> Self {
        if dy < -9 || dy > 9 {
            panic!("Shift amount out of range");
        }
        if dy <= 0 {
            // north
            self.shr(-dy as usize)
        } else {
            self.shl(dy as usize)
        }
    }

    /// Shift the bit set pattern horizontally.
    ///
    /// This shifts the bit set right if `dx < 0`, otherwise left.
    ///
    /// # Panics
    /// This will panic if the shift amount is out of range (abs(dx) > 9).
    ///
    #[inline(always)]
    pub const fn shift_along_rank(self, dx: i32) -> Self {
        if dx < -9 || dx > 9 {
            panic!("Shift amount out of range");
        }
        if dx <= 0 {
            self.shift_east(-dx as usize)
        } else {
            self.shift_west(dx as usize)
        }
    }

    /// Shift the bit set pattern right (east).
    ///
    /// # Panics
    /// Panics if the shift amount is too large.
    /// Caller should make sure that `abs(dx) <= 9`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use haitaka_types::*;
    /// let bb1 = bitboard! {
    ///     . . . . . . X X X
    ///     . . . . . . X . X
    ///     . . . . . . X X X
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     X X X . . . . . .
    ///     X . X . . . . . .
    ///     X X X . . . . . .
    /// };
    /// let bb2 = bitboard! {
    ///     . . . . . . . X X
    ///     . . . . . . . X .
    ///     . . . . . . . X X
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . X X X . . . . .
    ///     . X . X . . . . .
    ///     . X X X . . . . .
    /// };
    /// assert_eq!(bb1.shift_east(1), bb2);
    /// ```
    #[inline(always)]
    pub const fn shift_east(self, dx: usize) -> Self {
        BitBoard(self.0 >> (9 * dx))
    }

    /// Shift the bit set pattern left (west).
    ///
    /// # Panics
    /// Panics if the shift amount is too large.
    /// Caller should make sure that `abs(dx) <= 9`.
    ///
    /// # Example
    ///
    /// ```
    /// # use haitaka_types::*;
    /// let bb1 = bitboard! {
    ///     . . . . . . X X X
    ///     . . . . . . X . X
    ///     . . . . . . X X X
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     X X X . . . . . .
    ///     X . X . . . . . .
    ///     X X X . . . . . .
    /// };
    /// let bb2 = bitboard! {
    ///     . . . . . X X X .
    ///     . . . . . X . X .
    ///     . . . . . X X X .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     X X . . . . . . .
    ///     . X . . . . . . .
    ///     X X . . . . . . .
    /// };
    /// assert_eq!(bb1.shift_west(1), bb2);
    #[inline(always)]
    pub const fn shift_west(self, dx: usize) -> Self {
        BitBoard((self.0 << (9 * dx)) & BitBoard::BOARD_MASK)
    }

    /// Shift bit set pattern so that square 'from' is mapped to square 'to'.
    ///
    /// This maps square `from` to square `to` and applies the same translation
    /// to all other squares.
    ///
    /// # Examples
    ///
    /// ```
    /// use haitaka_types::*;
    /// let bb = Square::A1.bitboard();
    /// let shifted = bb.shift(Square::B1, Square::G2);
    /// assert_eq!(shifted, Square::F2.bitboard());
    /// ```
    pub const fn shift(self, from: Square, to: Square) -> Self {
        let dx = to.file() as i32 - from.file() as i32;
        let dy = to.rank() as i32 - from.rank() as i32;

        self.shift_along_file(dy).shift_along_rank(dx)
    }
}

//
// Core implementation
//
impl BitBoard {
    // TODO: It may be possible to optimize the code a bit by only using
    // BOARD_MASK during comparisons, so not when called BitBoard::new.
    // This would make it possible to get rid of BitBoard::new.

    /// Mask to select only the first 81 bits used in a the board position.
    pub const BOARD_MASK: u128 = (1 << Square::NUM) - 1;

    /// Create a new `BitBoard`.
    ///
    /// Note that `BitBoard(value)`` and `BitBoard::new(value)` behave differently.
    /// `BitBoard(value)`` does not mask out the unused high bits, while
    /// `BitBoard::new(value)` does ensure the high bits are set to zero. In some
    /// internal functions we do want to use the full `u128` bit set.
    #[inline(always)]
    pub const fn new(value: u128) -> Self {
        Self(value & Self::BOARD_MASK)
    }

    /// An empty bitboard.
    ///
    /// # Examples
    ///
    /// ```
    /// # use haitaka_types::*;
    /// assert_eq!(BitBoard::EMPTY, bitboard! {
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . * . . . * . .
    ///     . . . . . . . . .
    ///     . . . . * . . . .
    ///     . . . . . . . . .
    ///     . . * . . . * . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    /// });
    /// ```
    pub const EMPTY: Self = Self(0);

    /// A bitboard containing all squares.
    ///
    /// # Examples
    ///
    /// ```
    /// # use haitaka_types::*;
    /// assert_eq!(BitBoard::FULL, bitboard! {
    ///     X X X X X X X X X
    ///     X X X X X X X X X
    ///     X X X X X X X X X
    ///     X X X X X X X X X
    ///     X X X X X X X X X
    ///     X X X X X X X X X
    ///     X X X X X X X X X
    ///     X X X X X X X X X
    ///     X X X X X X X X X
    /// });
    /// ```
    pub const FULL: Self = Self::new(!0);

    /// The edges on the board.
    ///
    /// # Examples
    ///
    /// ```
    /// # use haitaka_types::*;
    /// assert_eq!(BitBoard::EDGES, bitboard! {
    ///     X X X X X X X X X
    ///     X . . . . . . . X
    ///     X . . . . . . . X
    ///     X . . . . . . . X
    ///     X . . . . . . . X
    ///     X . . . . . . . X
    ///     X . . . . . . . X
    ///     X . . . . . . . X
    ///     X X X X X X X X X
    /// });
    /// ```
    pub const EDGES: Self = Self::__EDGES;
    const __EDGES: Self = bitboard! {
        X X X X X X X X X
        X . . . . . . . X
        X . . . . . . . X
        X . . . . . . . X
        X . . . . . . . X
        X . . . . . . . X
        X . . . . . . . X
        X . . . . . . . X
        X X X X X X X X X
    };

    /// The non-edges of the board.
    pub const INNER: Self = Self::__INNER;
    const __INNER: Self = bitboard! {
        . . . . . . . . .
        . X X X X X X X .
        . X X X X X X X .
        . X X X X X X X .
        . X X X X X X X .
        . X X X X X X X .
        . X X X X X X X .
        . X X X X X X X .
        . . . . . . . . .
    };

    /// The corners of the board.
    ///
    /// # Examples
    ///
    /// ```
    /// # use haitaka_types::*;
    /// assert_eq!(BitBoard::CORNERS, bitboard! {
    ///     X . . . . . . . X
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     X . . . . . . . X
    /// });
    /// ```
    pub const CORNERS: Self = Self::__CORNERS;
    const __CORNERS: Self = bitboard! {
        X . . . . . . . X
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        X . . . . . . . X
    };

    // The 9x9 board makes it a bit more complicated to flip files and ranks.
    // The usual bag of bit hacks usually cannot be simply applied:
    // https://www.chessprogramming.org/Flipping_Mirroring_and_Rotating#Horizontal

    /// Flip the bitboard's files.
    ///
    /// This mirrors the bitboard vertically in file 5.
    ///
    /// # Examples
    /// ```
    /// # use haitaka_types::*;
    /// let bb = bitboard! {
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . X X X . . . .
    ///     . . X . X X . . .
    ///     . . X X X X . . .
    ///     . . X . X . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    /// };
    /// assert_eq!(bb.flip_files(), bitboard! {
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . X X X . .
    ///     . . . X X . X . .
    ///     . . . X X X X . .
    ///     . . . . X . X . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    /// });
    /// ```
    #[inline(always)]
    pub const fn flip_files(self) -> Self {
        const FILE_MASKS: [u128; 9] = [
            0x1FF,       // File 1
            0x1FF << 9,  // File 2
            0x1FF << 18, // File 3
            0x1FF << 27, // File 4
            0x1FF << 36, // File 5
            0x1FF << 45, // File 6
            0x1FF << 54, // File 7
            0x1FF << 63, // File 8
            0x1FF << 72, // File 9
        ];

        Self::new(
            ((self.0 & FILE_MASKS[0]) << 72)
                | ((self.0 & FILE_MASKS[1]) << 54)
                | ((self.0 & FILE_MASKS[2]) << 36)
                | ((self.0 & FILE_MASKS[3]) << 18)
                | (self.0 & FILE_MASKS[4])
                | ((self.0 & FILE_MASKS[5]) >> 18)
                | ((self.0 & FILE_MASKS[6]) >> 36)
                | ((self.0 & FILE_MASKS[7]) >> 54)
                | ((self.0 & FILE_MASKS[8]) >> 72),
        )
    }

    /// Flip the bitboard's ranks.
    ///
    /// This mirrors the bitboard horizontally in rank E.
    ///
    /// # Examples
    /// ```
    /// # use haitaka_types::*;
    /// let bb = bitboard! {
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . X X X . . . .
    ///     . . X . X X . . .
    ///     . . X X X X . . .
    ///     . . X . X . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    /// };
    /// assert_eq!(bb.flip_ranks(), bitboard! {
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . X . X . . . .
    ///     . . X X X X . . .
    ///     . . X . X X . . .
    ///     . . X X X . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    /// });
    /// ```
    #[inline(always)]
    pub const fn flip_ranks(self) -> Self {
        const RANK_ONE: u128 = 0x1008040201008040201;

        const RANK_MASKS: [u128; 9] = [
            RANK_ONE,
            RANK_ONE << 1,
            RANK_ONE << 2,
            RANK_ONE << 3,
            RANK_ONE << 4,
            RANK_ONE << 5,
            RANK_ONE << 6,
            RANK_ONE << 7,
            RANK_ONE << 8,
        ];

        Self::new(
            ((self.0 & RANK_MASKS[0]) << 8)
                | ((self.0 & RANK_MASKS[1]) << 6)
                | ((self.0 & RANK_MASKS[2]) << 4)
                | ((self.0 & RANK_MASKS[3]) << 2)
                | (self.0 & RANK_MASKS[4])
                | ((self.0 & RANK_MASKS[5]) >> 2)
                | ((self.0 & RANK_MASKS[6]) >> 4)
                | ((self.0 & RANK_MASKS[7]) >> 6)
                | ((self.0 & RANK_MASKS[8]) >> 8),
        )
    }

    /// Rotate this Bitboard 180 degrees.
    ///
    /// This maps bits 0..81 to bits 81..0, reversing the bits and rotating the board.
    ///
    /// # Examples
    ///
    /// ```
    /// # use haitaka_types::*;
    /// let bb = bitboard! {
    ///     X . . . . . . . .
    ///     . . . . . . . . .
    ///     . . X X X . . . .
    ///     . . X . X X . . .
    ///     . . X X X X . . .
    ///     . . X . X . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . X .
    ///     . . . . . . . . .
    /// };
    /// let rr = bitboard! {
    ///     . . . . . . . . .
    ///     . X . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . X . X . .
    ///     . . . X X X X . .
    ///     . . . X X . X . .
    ///     . . . . X X X . .
    ///     . . . . . . . . .
    ///     . . . . . . . . X
    /// };
    /// assert_eq!(bb.rotate(), rr);
    /// assert_eq!(rr.rotate(), bb);
    /// assert_eq!(bb.flip_files().flip_ranks(), rr);
    /// assert_eq!(bb.flip_ranks().flip_files(), rr);
    /// ```
    #[inline(always)]
    pub const fn rotate(self) -> Self {
        Self(self.0.reverse_bits() >> (128 - Square::NUM))
    }

    /// Reverse the bits of this bitboard.
    ///
    /// Note: This function does not shift the board. Bit 0 becomes bit 127 and vice-versa.
    #[inline(always)]
    pub const fn rev(self) -> Self {
        Self(self.0.reverse_bits())
    }

    /// Return the count of bits set to 1.
    ///
    /// This is an alias for [`BitBoard::count_ones`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use haitaka_types::*;
    /// assert_eq!(BitBoard::EMPTY.len(), 0);
    /// let bb = bitboard! {
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . X X X . . . .
    ///     . . X . X X . . .
    ///     . . X X X X . . .
    ///     . . X . X . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    /// };
    /// assert_eq!(bb.len(), 12);
    /// ```
    #[inline(always)]
    pub const fn len(self) -> u32 {
        self.0.count_ones()
    }

    /// Returns the number of ones in the binary representation of `self`.
    #[inline(always)]
    pub const fn count_ones(self) -> u32 {
        self.0.count_ones()
    }

    /// Returns the number of zeros in the binary representation of `self``.
    ///
    /// Warning: This discounts the upper 47 bits of the backing `u128` which
    /// are assumed to be zero.
    #[inline(always)]
    pub const fn count_zeros(self) -> u32 {
        self.0.count_zeros() - 47
    }

    /// Check if a [`Square`] is set.
    ///
    /// # Examples
    ///
    /// ```
    /// # use haitaka_types::*;
    /// let bb = bitboard! {
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . X X X . . . .
    ///     . . X . X X . . .
    ///     . . X X X X . . .
    ///     . . X . X . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    /// };
    /// assert!(bb.has(Square::C5));
    /// assert!(!bb.has(Square::D6));
    /// assert!(bb.has(Square::F5));
    /// assert!(!bb.has(Square::F6));
    /// assert!(bb.has(Square::F7));
    /// ```
    #[inline(always)]
    pub const fn has(self, square: Square) -> bool {
        // Warning: This is an optimized version of `has`
        // which relies on the file-major mapping of squares to bits.
        // Changing that layout will break this function.
        self.0 & (1u128 << square as usize) != 0
    }

    /// Remove a square from this [`BitBoard`].
    ///
    /// If the bitboard doesn't contain the square, this
    /// simply returns a copy of the original bitboard.
    ///
    /// # Examples
    ///
    /// ```
    /// use haitaka_types::*;
    /// let bb = Square::E5.bitboard();
    /// let ff = bb.rm(Square::E5);
    /// assert_eq!(ff, BitBoard::EMPTY);
    /// ```
    #[inline(always)]
    pub const fn rm(self, square: Square) -> Self {
        self.bitand(square.bitboard().not())
    }

    /// Check if a bitboard contains no squares in common with another.
    ///
    /// Returns true iff the intersection of the two bitboards is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use haitaka_types::*;
    /// let bb_a = bitboard! {
    ///     X X X . . . . . .
    ///     X . X X . . . . .
    ///     X X X X . . . . .
    ///     X . X . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    /// };
    /// let bb_b = bitboard! {
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . X X X . .
    ///     . . . . X . X X .
    ///     . . . . X X X X .
    ///     . . . . X . X . .
    /// };
    /// assert!(bb_a.is_disjoint(bb_b));
    /// ```
    #[inline(always)]
    pub const fn is_disjoint(self, other: BitBoard) -> bool {
        self.0 & other.0 == Self::EMPTY.0
    }

    /// Check if a bitboard is a subset of another.
    ///
    /// # Examples
    ///
    /// ```
    /// # use haitaka_types::*;
    /// let big = bitboard! {
    ///     . . . . . . . . .
    ///     . X X X X X . . .
    ///     . X X X X X X . .
    ///     . X X . X X X . .
    ///     . X X X X X X . .
    ///     . X X X X X . . .
    ///     . X X . X X . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    /// };
    /// let small = bitboard! {
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . X X X . . . .
    ///     . . X . X X . . .
    ///     . . X X X X . . .
    ///     . . X . X . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    /// };
    /// assert!(small.is_subset(small));
    /// assert!(small.is_subset(big));
    /// assert!(!big.is_subset(small));
    /// ```
    #[inline(always)]
    pub const fn is_subset(self, other: BitBoard) -> bool {
        other.0 & self.0 == self.0
    }

    /// Check if a bitboard is a superset of another.
    ///
    /// # Examples
    ///
    /// ```
    /// # use haitaka_types::*;
    /// let bb = bitboard! {
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . X X X . . . .
    ///     . . X . X X . . .
    ///     . . X X X X . . .
    ///     . . X . X . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    /// };
    /// let superset = bitboard! {
    ///     . . . . . . . . .
    ///     . X X X X X . . .
    ///     . X X X X X X . .
    ///     . X X . X X X . .
    ///     . X X X X X X . .
    ///     . X X X X X . . .
    ///     . X X . X X . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    /// };
    /// assert!(superset.is_superset(bb));
    /// ```
    #[inline(always)]
    pub const fn is_superset(self, other: BitBoard) -> bool {
        self.0 & other.0 == other.0
    }

    /// Checks if the [`BitBoard`] is empty.
    ///
    /// # Examples
    /// ```
    /// # use haitaka_types::*;
    /// assert!(BitBoard::EMPTY.is_empty());
    /// let bb = bitboard! {
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . X X X . . . .
    ///     . . X . X X . . .
    ///     . . X X X X . . .
    ///     . . X . X . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    /// };
    /// assert!(!bb.is_empty());
    /// assert!(BitBoard::new(0).is_empty());
    /// ```
    #[inline(always)]
    pub const fn is_empty(self) -> bool {
        self.0 == Self::EMPTY.0
    }

    /// Grabs the first square if the bitboard is not empty.
    ///
    /// "First" means the first square when scanning from A1 to I9.
    ///
    /// # Examples
    /// ```
    /// # use haitaka_types::*;
    /// assert!(BitBoard::EMPTY.next_square().is_none());
    /// let bb = bitboard! {
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . X X X . . . .
    ///     . . X . X X . . .
    ///     . . X X X X . . .
    ///     . . X . X . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    /// };
    /// assert_eq!(bb.next_square(), Some(Square::D4));
    /// ```
    #[inline(always)]
    pub const fn next_square(self) -> Option<Square> {
        if self.0 > 0 {
            Some(Square::index_const(self.0.trailing_zeros() as usize))
        } else {
            None
        }
    }

    /// Iterate over the squares in the bitboard, ordered by square.
    ///
    /// The order follows the default enumeration of [`Square::ALL`],
    /// which is the file-major order of A1, B1, C1, ... G9, H9, I9.
    ///
    /// # Examples
    ///
    /// ```
    /// # use haitaka_types::*;
    /// let bb = BitBoard::FULL;
    /// let squares = &Square::ALL;
    /// for (s1, &s2) in bb.iter().zip(squares) {
    ///     assert_eq!(s1, s2);
    /// }
    /// ```
    #[inline(always)]
    pub fn iter(self) -> BitBoardIter {
        BitBoardIter(self)
    }

    /// Iterate over all subsets of a bitboard.
    ///
    /// Subsets are produced in lexicographic order. Each subset is greater than the last.
    ///
    /// ```
    /// # use haitaka_types::*;
    /// let bb = bitboard! {
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . X X X . . . .
    ///     . . X . X X . . .
    ///     . . X X X X . . .
    ///     . . X . X . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    /// };
    /// for subset in bb.iter_subsets() {
    ///     assert!(subset.is_subset(bb));
    /// }
    /// ```
    #[inline(always)]
    pub fn iter_subsets(self) -> BitBoardSubsetIter {
        BitBoardSubsetIter {
            set: self,
            subset: Self::EMPTY,
            finished: false,
        }
    }
}

impl IntoIterator for BitBoard {
    type Item = Square;

    type IntoIter = BitBoardIter;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl FromIterator<Square> for BitBoard {
    fn from_iter<T: IntoIterator<Item = Square>>(iter: T) -> Self {
        iter.into_iter()
            .fold(Self::EMPTY, |bb, sq| bb | sq.bitboard())
    }
}

/// An iterator over the squares of a bitboard.
///
/// This `struct` is created by [`BitBoard::iter`]. See its documentation for more.
pub struct BitBoardIter(BitBoard);

impl Iterator for BitBoardIter {
    type Item = Square;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let square = self.0.next_square();
        if let Some(square) = square {
            self.0 ^= square.bitboard();
        }
        square
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl ExactSizeIterator for BitBoardIter {
    #[inline(always)]
    fn len(&self) -> usize {
        self.0.len() as usize
    }
}

/// An iterator over the subsets of a bitboard.
///
/// This `struct` is created by [`BitBoard::iter_subsets`]. See its documentation for more.
pub struct BitBoardSubsetIter {
    set: BitBoard,
    subset: BitBoard,
    finished: bool,
}

impl Iterator for BitBoardSubsetIter {
    type Item = BitBoard;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        let current = self.subset;
        // Carry-Rippler trick to enumerate all subsets of a set.
        // https://www.chessprogramming.org/Traversing_Subsets_of_a_Set#All_Subsets_of_any_Set
        self.subset.0 = self.subset.0.wrapping_sub(self.set.0) & self.set.0;
        self.finished = self.subset.is_empty();
        Some(current)
    }
}

/// [`BitBoard`] literal macro.
///
/// The macro takes as input a visual rendering of the Shogi board from the
/// perspective of the Sente player. This is the way Shogi diagrams are usually
/// displayed with square I9 in the left bottom (south-west) corner and square A1
/// in the right top (north-east) corner.
///
/// The macro reads dot (.) or star (*) as empty squares and X as occupied.
/// Other characters will lead to a compilation error. The '*' is used to indicate
/// special empty squares (for instance, the piece position in an attack pattern).
///
/// Internally we layout the board by files: file 1 (squares A1, B1, C1 ...)
/// corresponds to the least significant bits in the underlying u128 value, followed
/// by file 2 (A2, B2, ... I2), up to file 9.
///
/// # Example
///
/// ```
/// # use haitaka_types::*;
/// let bb = bitboard! {
///     . . . X . . . . .
///     . . . X . . . . .
///     . . . X . . . . .
///     . . . X . . . . .
///     . . . X . . . . .
///     X X X . X X X X X
///     . . . X . . . . .
///     . . . X . . . . .
///     . . . X . . . . .
/// };
/// assert_eq!(bb, File::Six.bitboard() ^ Rank::F.bitboard());
/// ```
#[macro_export]
macro_rules! bitboard {
    (   $a9:tt $a8:tt $a7:tt $a6:tt $a5:tt $a4:tt $a3:tt $a2:tt $a1:tt
        $b9:tt $b8:tt $b7:tt $b6:tt $b5:tt $b4:tt $b3:tt $b2:tt $b1:tt
        $c9:tt $c8:tt $c7:tt $c6:tt $c5:tt $c4:tt $c3:tt $c2:tt $c1:tt
        $d9:tt $d8:tt $d7:tt $d6:tt $d5:tt $d4:tt $d3:tt $d2:tt $d1:tt
        $e9:tt $e8:tt $e7:tt $e6:tt $e5:tt $e4:tt $e3:tt $e2:tt $e1:tt
        $f9:tt $f8:tt $f7:tt $f6:tt $f5:tt $f4:tt $f3:tt $f2:tt $f1:tt
        $g9:tt $g8:tt $g7:tt $g6:tt $g5:tt $g4:tt $g3:tt $g2:tt $g1:tt
        $h9:tt $h8:tt $h7:tt $h6:tt $h5:tt $h4:tt $h3:tt $h2:tt $h1:tt
        $i9:tt $i8:tt $i7:tt $i6:tt $i5:tt $i4:tt $i3:tt $i2:tt $i1:tt
    ) => {
        $crate::bitboard! { @__inner
            $a1 $b1 $c1 $d1 $e1 $f1 $g1 $h1 $i1
            $a2 $b2 $c2 $d2 $e2 $f2 $g2 $h2 $i2
            $a3 $b3 $c3 $d3 $e3 $f3 $g3 $h3 $i3
            $a4 $b4 $c4 $d4 $e4 $f4 $g4 $h4 $i4
            $a5 $b5 $c5 $d5 $e5 $f5 $g5 $h5 $i5
            $a6 $b6 $c6 $d6 $e6 $f6 $g6 $h6 $i6
            $a7 $b7 $c7 $d7 $e7 $f7 $g7 $h7 $i7
            $a8 $b8 $c8 $d8 $e8 $f8 $g8 $h8 $i8
            $a9 $b9 $c9 $d9 $e9 $f9 $g9 $h9 $i9
        }
    };
    (@__inner $($occupied:tt)*) => {{
        const BITBOARD: $crate::BitBoard = {
            let mut index = 0;
            let mut bitboard = $crate::BitBoard::EMPTY;
            $(
                if $crate::bitboard!(@__square $occupied) {
                    bitboard.0 |= 1 << index;
                }
                index += 1;
            )*
            let _ = index;
            bitboard
        };
        BITBOARD
    }};
    (@__square X) => { true };
    (@__square .) => { false };
    (@__square *) => { false };
    (@__square $token:tt) => {
        compile_error!(
            concat!(
                "Expected only `X` or `.` tokens, found `",
                stringify!($token),
                "`"
            )
        )
    };
    ($($token:tt)*) => {
        compile_error!("Expected 81 squares")
    };
}
pub use bitboard;

impl core::fmt::Debug for BitBoard {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            write!(f, "bitboard! {{")?;
            for &rank in Rank::ALL.iter() {
                write!(f, "\n   ")?;
                for &file in File::ALL.iter().rev() {
                    if self.has(Square::new(file, rank)) {
                        write!(f, " X")?;
                    } else {
                        write!(f, " .")?;
                    }
                }
            }
            write!(f, "\n}}")?;
            Ok(())
        } else {
            write!(f, "BitBoard({:#018X})", self.0)
        }
    }
}
