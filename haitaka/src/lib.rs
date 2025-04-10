//#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![doc = include_str!("../README.md")]

use haitaka_types::*;

pub use bitboard::*;
pub use color::*;
pub use file::*;
pub use piece::*;
pub use rank::*;
pub use shogi_move::*;
pub use sliders::*;
pub use square::*;

pub mod attacks;
pub mod board;
pub mod slider_moves;

pub use attacks::*;
pub use board::*;
pub use slider_moves::*;
