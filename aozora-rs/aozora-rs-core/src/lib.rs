//! # 使い方
//! preludeの中の

mod azerror;
mod deco;
mod meta;
mod nihongo;

pub mod retokenizer;
pub mod scopenizer;
pub mod tokenizer;

use winnow::LocatingSlice;

pub type Input<'s> = LocatingSlice<&'s str>;
pub type Span = std::ops::Range<usize>;

pub use crate::deco::*;

pub use crate::azerror::{AZResult, AZResultC};

pub use crate::meta::{AozoraMeta, parse_meta};

pub use crate::retokenizer::{Retokenized, retokenize};
pub use crate::scopenizer::{ScopeKind, Scopenized, scopenize};
pub use crate::tokenizer::{AozoraTokenKind, Tokenized, tokenize};
