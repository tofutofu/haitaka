#![allow(missing_docs)]

mod common;
#[cfg(not(feature = "qugiy"))]
mod magic;

pub use common::*;
#[cfg(not(feature = "qugiy"))]
pub use magic::*;
