use crate::prelude::*;
use crate::tokenizer::prelude::*;
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub enum AozoraTokenKind<'s> {
    Command(Note<'s>),
    Ruby(&'s str),
    RubyDelimiter,
    Odoriji(Odoriji),
    Text(Cow<'s, str>),
    Br,
}

#[derive(Debug, Clone)]
pub struct AozoraToken<'s> {
    pub kind: AozoraTokenKind<'s>,
    pub span: Span,
}
