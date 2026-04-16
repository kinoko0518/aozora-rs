use winnow::{Parser, combinator::alt, error::ContextError};

use crate::tokenizer::note::backref::BackRef;
use crate::tokenizer::note::backref::backref;
use crate::tokenizer::note::multiline::multiline;
use crate::tokenizer::note::sandwiched::sandwiched;
use crate::tokenizer::note::single::single;
use crate::tokenizer::note::wholeline::{WholeLine, wholeline};
use crate::tokenizer::*;
use crate::*;

pub mod backref;
pub mod definitions;
#[macro_use]
pub mod multiline;
#[macro_use]
pub mod sandwiched;
pub mod single;
pub mod wholeline;

#[derive(Debug, Clone)]
pub enum Note<'s> {
    BackRef(BackRef<'s>),
    Sandwiched(Sandwiched),
    Multiline(MultiLine),
    Single(Single<'s>),
    WholeLine(WholeLine),
    Unknown(&'s str),
}

pub trait SandwichedBegin<E> {
    fn do_match(&self, rhs: &E) -> bool;
}

type RNote<'s> = Result<Note<'s>, ContextError>;

/// 注記にマッチするパーサーです。
pub fn command<'s>(input: &mut Input<'s>) -> RNote<'s> {
    alt((
        backref.map(Note::BackRef),
        sandwiched.map(Note::Sandwiched),
        multiline.map(Note::Multiline),
        wholeline.map(Note::WholeLine),
        single.map(Note::Single),
    ))
    .parse_next(input)
}
