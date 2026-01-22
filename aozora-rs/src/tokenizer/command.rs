use winnow::{Parser, combinator::alt, error::ContextError};

use crate::{
    Input,
    tokenizer::{
        Span,
        command::{
            backref::BackRefNote,
            multiline::{MultiLine, multiline},
            sandwiched::Sandwiched,
        },
    },
};

macro_rules! impl_sandwiched {
    ($generics:ident, $target_struct:ident, $target_variant:ident) => {
        impl SandwichedBegin<$generics> for $target_struct {
            fn effect_range(&self, rhs: &$generics) -> Option<Span> {
                if let $generics::$target_variant(s) = rhs {
                    Some((self.span.end + 1)..(s.start - 1))
                } else {
                    None
                }
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

pub enum Note<'s> {
    BackRef(BackRefNote<'s>),
    Sandwiched(Sandwiched),
    Multiline(MultiLine),
}

trait SandwichedBegin<E> {
    fn effect_range(&self, rhs: &E) -> Option<Span>;
    fn do_match(&self, rhs: &E) -> bool {
        self.effect_range(rhs).is_some()
    }
}

type RNote<'s> = Result<Note<'s>, ContextError>;

pub fn command<'s>(input: &mut Input) -> RNote<'s> {
    alt((multiline.map(|s| Note::Multiline(s)),)).parse_next(input)
}
