use crate::tokenizer::*;
use crate::*;
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub enum AozoraTokenKind<'s> {
    Note(Note<'s>),
    Ruby(&'s str),
    RubyDelimiter,
    Odoriji(Odoriji),
    Text(Cow<'s, str>),
    Br,
}

#[derive(Debug, Clone)]
pub struct Tokenized<'s> {
    pub kind: AozoraTokenKind<'s>,
    pub span: Span,
}
