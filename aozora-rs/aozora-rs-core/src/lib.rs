mod deco;
mod error;
mod meta;
mod nihongo;

pub mod retokenizer;
pub mod scopenizer;
pub mod tokenizer;

use winnow::LocatingSlice;

pub type Input<'s> = LocatingSlice<&'s str>;
pub type Span = std::ops::Range<usize>;
pub type WinnowError = ();

pub use crate::deco::*;

pub use crate::error::*;

pub use crate::meta::*;
pub use crate::retokenizer::*;
pub use crate::scopenizer::*;
pub use crate::tokenizer::*;
