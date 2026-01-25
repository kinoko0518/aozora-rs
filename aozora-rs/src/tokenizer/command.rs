use winnow::{Parser, combinator::alt, error::ContextError};

use crate::prelude::*;
use crate::tokenizer::command::backref::backref;
use crate::tokenizer::command::multiline::multiline;
use crate::tokenizer::command::sandwiched::sandwiched;
use crate::tokenizer::command::single::single;
use crate::tokenizer::prelude::*;

pub use crate::tokenizer::command::backref::BackRef;

macro_rules! impl_sandwiched {
    ($generics:ident, $target_struct:ident, $target_variant:ident) => {
        impl SandwichedBegin<$generics> for $target_struct {
            fn do_match(&self, rhs: &$generics) -> bool {
                matches!(rhs, $generics::$target_variant)
            }
        }
    };
}

macro_rules! impl_sandwiched_ignore {
    ($generics:ident, $target_struct:ident, $target_variant:ident) => {
        impl SandwichedBegin<$generics> for $target_struct {
            fn do_match(&self, rhs: &$generics) -> bool {
                matches!(rhs, $generics::$target_variant(_))
            }
        }
    };
}

pub mod backref;
pub mod definitions;
#[macro_use]
pub mod multiline;
#[macro_use]
pub mod sandwiched;
pub mod single;

#[derive(Debug, Clone)]
pub enum Note<'s> {
    BackRef(BackRef<'s>),
    Sandwiched(Sandwiched),
    Multiline(MultiLine),
    Single(Single<'s>),
    Unknown(&'s str),
}

pub trait SandwichedBegin<E> {
    fn do_match(&self, rhs: &E) -> bool;
}

type RNote<'s> = Result<Note<'s>, ContextError>;

pub fn command<'s>(input: &mut Input<'s>) -> RNote<'s> {
    alt((
        multiline.map(|m| Note::Multiline(m)),
        single.map(|m| Note::Single(m)),
        backref.map(|m| Note::BackRef(m)),
        sandwiched.map(|m| Note::Sandwiched(m)),
    ))
    .parse_next(input)
}
