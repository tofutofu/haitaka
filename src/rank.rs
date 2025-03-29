//! This module defines the Rank enum which represents the ranks (rows) on a Shogi board.
//!
//! Shogi ranks are indicated by the letters abdefghi or by Kanji numerals 一二三四五六七八九.
//! Here we use capital letters to indicate the ranks: Rank::A .. Rank::I.
//!  

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

// north and south, given the usual board diagrams

const NORTH_A: BitBoard = BitBoard::EMPTY;
const NORTH_B: BitBoard = RANK_A;
const NORTH_C: BitBoard = NORTH_B.bitor(RANK_B);
const NORTH_D: BitBoard = NORTH_C.bitor(RANK_C);
const NORTH_E: BitBoard = NORTH_D.bitor(RANK_D);
const NORTH_F: BitBoard = NORTH_E.bitor(RANK_E);
const NORTH_G: BitBoard = NORTH_F.bitor(RANK_F);
const NORTH_H: BitBoard = NORTH_G.bitor(RANK_G);
const NORTH_I: BitBoard = NORTH_H.bitor(RANK_H);

const SOUTH_I: BitBoard = BitBoard::EMPTY;
const SOUTH_H: BitBoard = RANK_I;
const SOUTH_G: BitBoard = SOUTH_H.bitor(RANK_H);
const SOUTH_F: BitBoard = SOUTH_G.bitor(RANK_G);
const SOUTH_E: BitBoard = SOUTH_F.bitor(RANK_F);
const SOUTH_D: BitBoard = SOUTH_E.bitor(RANK_E);
const SOUTH_C: BitBoard = SOUTH_D.bitor(RANK_D);
const SOUTH_B: BitBoard = SOUTH_C.bitor(RANK_C);
const SOUTH_A: BitBoard = SOUTH_B.bitor(RANK_B);

/// Get the no-fly-zones for a piece.
/// Returns a BitBoard where a piece of the given color may not
/// be dropped.
#[inline(always)]
pub const fn no_fly_zone(color: Color, piece: Piece) -> BitBoard {
    match piece {
        Piece::Pawn | Piece::Lance => {
            if color as usize == Color::White as usize {
                RANK_I
            } else {
                RANK_A
            }
        }
        Piece::Knight => {
            if color as usize == Color::White as usize {
                RANK_I.bitor(RANK_H)
            } else {
                RANK_A.bitor(RANK_B)
            }
        }
        _ => BitBoard::EMPTY,
    }
}

/// Returns a BitBoard representing all squares where a piece may
/// be dropped. This is the inverse of `no_fly_zone`.
pub const fn drop_zone(color: Color, piece: Piece) -> BitBoard {
    match piece {
        Piece::Pawn | Piece::Lance => {
            if color as usize == Color::White as usize {
                NORTH_I
            } else {
                SOUTH_A
            }
        }
        Piece::Knight => {
            if color as usize == Color::White as usize {
                NORTH_H
            } else {
                SOUTH_B
            }
        }
        _ => BitBoard::FULL,
    }
}

impl Rank {
    // TODO: Should these array be lifted out of the impl
    // to avoid code bloat?!

    /// Bitboards for the 9 ranks.
    pub const RANK: [BitBoard; Self::NUM] = [
        RANK_A, RANK_B, RANK_C, RANK_D, RANK_E, RANK_F, RANK_G, RANK_H, RANK_I,
    ];

    /// Cover all ranks "SOUTH" a given rank.
    /// "SOUTH" given the usual board diagrams.
    ///
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// assert_eq!(Rank::SOUTH[2], bitboard! {
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
    pub const SOUTH: [BitBoard; Self::NUM] = [
        SOUTH_A, SOUTH_B, SOUTH_C, SOUTH_D, SOUTH_E, SOUTH_F, SOUTH_G, SOUTH_H, SOUTH_I,
    ];

    /// Cover all ranks "NORTH" a given rank.
    /// "NORTH" from the point of view of Gote.
    ///
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// assert_eq!(Rank::NORTH[2], bitboard! {
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
    pub const NORTH: [BitBoard; Self::NUM] = [
        NORTH_A, NORTH_B, NORTH_C, NORTH_D, NORTH_E, NORTH_F, NORTH_G, NORTH_H, NORTH_I,
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
        Self::RANK[self as usize]
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

    /// Return the BitBoard for all ranks "north" of this rank.
    ///
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// assert_eq!(Rank::A.north(), BitBoard::EMPTY);
    /// assert_eq!(Rank::C.north(), bitboard!{
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
    #[inline(always)]
    pub const fn north(self) -> BitBoard {
        Self::NORTH[self as usize]
    }

    /// Return the BitBoard for all ranks "south" of this rank.
    ///  
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// assert_eq!(Rank::I.south(), BitBoard::EMPTY);
    /// assert_eq!(Rank::G.south(), bitboard!{
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
    pub const fn south(self) -> BitBoard {
        Self::SOUTH[self as usize]
    }

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
        match color {
            Color::White => self,
            Color::Black => self.flip(),
        }
    }
}
