mod command;

use std::borrow::Cow;

use aozora_rs_gaiji::whole_gaiji_to_char;
use winnow::{
    Parser,
    combinator::{alt, delimited, eof, opt, peek, repeat, repeat_till},
    error::ContextError,
    token::{any, take_until},
};

use crate::{Input, tokenizer::command::command};

pub use crate::tokenizer::command::*;

use crate::Span;

#[derive(Debug, Clone, Copy)]
pub struct Odoriji {
    pub has_dakuten: bool,
}

impl std::fmt::Display for Odoriji {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}〵", if self.has_dakuten { "〴" } else { "〳" })
    }
}

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

fn ruby<'s>(input: &mut Input<'s>) -> Result<&'s str, ContextError> {
    const END: char = '》';
    delimited('《', take_until(1.., END), END).parse_next(input)
}

fn odoriji<'s>(input: &mut Input<'s>) -> Result<Odoriji, ContextError> {
    ("／", opt('″'), "＼")
        .map(|(_, dakuten, _)| Odoriji {
            has_dakuten: dakuten.is_some(),
        })
        .parse_next(input)
}

fn special<'s>(input: &mut Input<'s>) -> Result<AozoraTokenKind<'s>, ContextError> {
    alt((
        '｜'.value(AozoraTokenKind::RubyDelimiter),
        '\n'.value(AozoraTokenKind::Br),
        delimited("［＃", command.map(|c| AozoraTokenKind::Command(c)), "］"),
        ruby.map(|r| AozoraTokenKind::Ruby(r)),
        odoriji.map(|o| AozoraTokenKind::Odoriji(o)),
    ))
    .parse_next(input)
}

fn take_until_special<'s>(input: &mut Input<'s>) -> Result<&'s str, ContextError> {
    let end = alt((peek(special).void(), eof.void()));
    repeat_till(1.., any, end)
        .map(|(s, _): ((), _)| s)
        .take()
        .parse_next(input)
}

pub fn tokenize<'s>(input: &mut Input<'s>) -> Result<Vec<AozoraToken<'s>>, ContextError> {
    let result: Vec<AozoraToken> = repeat(
        0..,
        alt((
            special,
            take_until_special
                .map(whole_gaiji_to_char)
                .map(AozoraTokenKind::Text),
        ))
        .with_span()
        .map(|(k, s)| AozoraToken { kind: k, span: s }),
    )
    .parse_next(input)?;
    Ok(result)
}
