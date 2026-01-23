use winnow::{Parser, combinator::alt, error::ContextError};

use crate::{
    Input,
    tokenizer::command::{
        backref::{BackRef, backref},
        multiline::multiline,
        sandwiched::sandwiched,
        single::single,
    },
};

pub use crate::tokenizer::command::{
    backref::BackRefKind, definitions::*, multiline::MultiLine, sandwiched::Sandwiched,
    single::Single,
};

macro_rules! impl_sandwiched {
    ($generics:ident, $target_struct:ident, $target_variant:ident) => {
        impl SandwichedBegin<$generics> for $target_struct {
            fn do_match(rhs: &$generics) -> bool {
                matches!(rhs, $generics::$target_variant)
            }
        }
    };
}

macro_rules! impl_sandwiched_ignore {
    ($generics:ident, $target_struct:ident, $target_variant:ident) => {
        impl SandwichedBegin<$generics> for $target_struct {
            fn do_match(rhs: &$generics) -> bool {
                matches!(rhs, $generics::$target_variant(_))
            }
        }
    };
}

mod backref;
mod definitions;
#[macro_use]
mod multiline;
#[macro_use]
mod sandwiched;
mod single;

#[derive(Debug, Clone, Copy)]
pub enum Note<'s> {
    BackRef(BackRef<'s>),
    Sandwiched(Sandwiched),
    Multiline(MultiLine),
    Single(Single),
}

trait SandwichedBegin<E> {
    fn do_match(rhs: &E) -> bool;
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
