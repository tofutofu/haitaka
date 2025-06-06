//! The [`Color`] enum
crate::helpers::simple_enum! {
    /// A side to move.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Color {
        /// The color white.
        White,
        /// The color black.
        Black
    }
}

crate::helpers::enum_char_conv! {
    Color, ColorParseError {
        White = 'w',
        Black = 'b'
    }
}

impl Color {
    /// Not. This can be used in const functions.
    #[inline(always)]
    pub const fn not(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

impl core::ops::Not for Color {
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self::Output {
        self.not()
    }
}
