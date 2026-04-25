#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

mod deco;
mod error;
mod meta;
mod nihongo;

pub mod retokenizer;
pub mod scopenizer;
pub mod tokenizer;

/// トークナイズに用いるパーサーが受け取るテキストの型です。
pub type Input<'s> = winnow::LocatingSlice<&'s str>;
/// トークナイザで切り出すスパン情報の型です。
pub type Span = std::ops::Range<usize>;
/// トークナイズに用いるパーサーに共通のエラー型です。
pub type WinnowError = ();

pub use crate::deco::*;

pub use crate::error::*;

pub use crate::meta::*;
pub use crate::retokenizer::*;
pub use crate::scopenizer::*;
pub use crate::tokenizer::*;
