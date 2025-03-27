use crate::*;


crate::helpers::simple_enum! {
    /// A rank (row) on a shogi board.
    /// 
    /// Ranks are indicated by letters or by Kanji numerals.
    /// Rank 'a' ("一") is the top-most rank board diagrams which are
    /// always shown from the perspective of the Sente player.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub enum Rank {
        /// The first rank.
        A,
        /// The second rank.
        B,
        /// The third rank.
        C,
        /// The fourth rank.
        D,
        /// The fifth rank.
        E,
        /// The sixth rank.
        F,
        /// The seventh rank.
        G,
        /// The eighth rank.
        H,
        /// The ninth rank.
        I
    }
}

crate::helpers::enum_char_conv! {
    Rank, RankParseError {
        A = 'a',
        B = 'b',
        C = 'c',
        D = 'd',
        E = 'e',
        F = 'f',
        G = 'g',
        H = 'h',
        I = 'i'
    }
}

// MASK corresponds to all set bits in Rank::A.
// Remember that the board is oriented so that File 1 corresponds with
// the LSB bits in a bitboard. This makes it a little less convenient
// to handle ranks.

const MASK: u128 = 0x1008040201008040201;
const RANK_A: BitBoard = BitBoard(MASK << 0);
const RANK_B: BitBoard = BitBoard(MASK << 1);
const RANK_C: BitBoard = BitBoard(MASK << 2);
const RANK_D: BitBoard = BitBoard(MASK << 3);
const RANK_E: BitBoard = BitBoard(MASK << 4);
const RANK_F: BitBoard = BitBoard(MASK << 5);
const RANK_G: BitBoard = BitBoard(MASK << 6);
const RANK_H: BitBoard = BitBoard(MASK << 7);
const RANK_I: BitBoard = BitBoard(MASK << 8);

// below from the point of view of Gote
const BELOW_A: BitBoard = BitBoard::EMPTY;
const BELOW_B: BitBoard = RANK_A;
const BELOW_C: BitBoard = BELOW_B.bitor(RANK_B);
const BELOW_D: BitBoard = BELOW_C.bitor(RANK_C);
const BELOW_E: BitBoard = BELOW_D.bitor(RANK_D);
const BELOW_F: BitBoard = BELOW_E.bitor(RANK_E);
const BELOW_G: BitBoard = BELOW_F.bitor(RANK_F);
const BELOW_H: BitBoard = BELOW_G.bitor(RANK_G);
const BELOW_I: BitBoard = BELOW_H.bitor(RANK_H);

// above from the point of view of Gote
const ABOVE_I: BitBoard = BitBoard::EMPTY;
const ABOVE_H: BitBoard = RANK_I;
const ABOVE_G: BitBoard = ABOVE_H.bitor(RANK_H);
const ABOVE_F: BitBoard = ABOVE_G.bitor(RANK_G);
const ABOVE_E: BitBoard = ABOVE_F.bitor(RANK_F);
const ABOVE_D: BitBoard = ABOVE_E.bitor(RANK_E);
const ABOVE_C: BitBoard = ABOVE_D.bitor(RANK_D);
const ABOVE_B: BitBoard = ABOVE_C.bitor(RANK_C);
const ABOVE_A: BitBoard = ABOVE_B.bitor(RANK_B);

impl Rank {

    /// Bitboards for the 9 ranks.
    pub const BB: [BitBoard; Self::NUM] = [
        RANK_A,
        RANK_B,
        RANK_C,
        RANK_D,
        RANK_E,
        RANK_F,
        RANK_G,
        RANK_H,
        RANK_I
    ];

    /// Cover all ranks "above" a given rank. 
    /// "Above" from the point of view of Gote.
    /// 
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// assert_eq!(Rank::ABOVE[2], bitboard! {
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     X X X X X X X X X
    ///     X X X X X X X X X
    ///     X X X X X X X X X
    ///     X X X X X X X X X
    ///     X X X X X X X X X
    ///     X X X X X X X X X
    /// });
    /// ```
    pub const ABOVE: [BitBoard; Self::NUM] = [
        ABOVE_A,
        ABOVE_B,
        ABOVE_C,
        ABOVE_D,
        ABOVE_E,
        ABOVE_F,
        ABOVE_G,
        ABOVE_H,
        ABOVE_I,        
    ];

    /// Cover all ranks "below" a given rank.
    /// "Below" from the point of view of Gote.
    /// 
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// assert_eq!(Rank::BELOW[2], bitboard! {
    ///     X X X X X X X X X
    ///     X X X X X X X X X
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    /// });
    /// ```
    pub const BELOW: [BitBoard; Self::NUM] = [
        BELOW_A,
        BELOW_B,
        BELOW_C,
        BELOW_D,
        BELOW_E,
        BELOW_F,
        BELOW_G,
        BELOW_H,
        BELOW_I,
    ];

    /// Get a bitboard with all squares on this rank set.
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// assert_eq!(Rank::A as usize, 0);
    /// assert_eq!(Rank::H.bitboard(), bitboard! {
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     X X X X X X X X X
    ///     . . . . . . . . .
    /// });
    /// ```
    #[inline(always)]
    pub const fn bitboard(self) -> BitBoard {
        Self::BB[self as usize]
    }

    /// Flip the rank.
    /// 
    /// This mirrors the rank in the fifth E rank.
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// assert_eq!(Rank::A.flip(), Rank::I);
    /// ```
    #[inline(always)]
    pub const fn flip(self) -> Self {
        Self::index_const(Self::I as usize - self as usize)
    }

    /// Ranks "below" this rank as seen from the perspective of color.
    /// 
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// assert_eq!(Rank::A.below(Color::White), BitBoard::EMPTY);
    /// assert_eq!(Rank::I.below(Color::Black), BitBoard::EMPTY);
    /// assert_eq!(Rank::C.below(Color::White), bitboard!{
    ///     X X X X X X X X X
    ///     X X X X X X X X X
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    /// });
    /// assert_eq!(Rank::C.below(Color::Black), bitboard!{
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     X X X X X X X X X
    ///     X X X X X X X X X
    ///     X X X X X X X X X
    ///     X X X X X X X X X
    ///     X X X X X X X X X
    ///     X X X X X X X X X
    /// });
    /// 
    /// ``````
    #[inline(always)]
    pub const fn below(self, color: Color) -> BitBoard {
        match color {
            Color::White => Self::BELOW[self as usize],
            Color::Black => Self::ABOVE[self as usize]
        }
    }

    /// Ranks "above" this rank as seen from the perspective of color.
    /// 
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// //assert_eq!(Rank::A.above(Color::Black), BitBoard::EMPTY);
    /// //assert_eq!(Rank::I.above(Color::White), BitBoard::EMPTY);
    /// assert_eq!(Rank::G.above(Color::Black), bitboard!{
    ///     X X X X X X X X X
    ///     X X X X X X X X X
    ///     X X X X X X X X X
    ///     X X X X X X X X X
    ///     X X X X X X X X X
    ///     X X X X X X X X X
    ///     . . . . . . . . .
    ///     . . . . . . . . .
    ///     . . . . . . . . . 
    /// });
    /// assert_eq!(Rank::G.above(Color::White), bitboard!{
    ///     . . . . . . . . . 
    ///     . . . . . . . . . 
    ///     . . . . . . . . . 
    ///     . . . . . . . . . 
    ///     . . . . . . . . . 
    ///     . . . . . . . . .
    ///     . . . . . . . . .  
    ///     X X X X X X X X X
    ///     X X X X X X X X X
    /// });
    /// ```
    #[inline(always)]
    pub const fn above(self, color: Color) -> BitBoard {
        match color {
            Color::White => Self::ABOVE[self as usize],
            Color::Black => Self::BELOW[self as usize]
        }
    }

    // TODO: Check how this is used.

    /// Get a rank relative to some color.
    /// This flips the rank if viewing from Black's perspective.
    /// 
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// assert_eq!(Rank::A.relative_to(Color::White), Rank::A);
    /// assert_eq!(Rank::A.relative_to(Color::Black), Rank::I);
    /// ```
    #[inline(always)]
    pub const fn relative_to(self, color: Color) -> Self {
        if let Color::White = color {
            self
        } else {
            self.flip()
        }
    }
}
