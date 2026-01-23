mod command;

use std::borrow::Cow;

use aozora_rs_gaiji::whole_gaiji_to_char;
use winnow::{
    Parser,
    combinator::{alt, delimited, eof, opt, peek, repeat, repeat_till},
    error::ContextError,
    token::{any, take_until},
};

use crate::{
    Input,
    tokenizer::command::{Note, command},
};

type Span = std::ops::Range<usize>;

#[derive(Debug, Clone, Copy)]
struct Odoriji {
    has_dakuten: bool,
}

#[derive(Debug, Clone)]
enum AozoraTokenKind<'s> {
    Command(Note<'s>),
    Ruby(&'s str),
    RubyDelimiter,
    Odoriji(Odoriji),
    Text(Cow<'s, str>),
}

#[derive(Debug, Clone)]
pub struct AozoraToken<'s> {
    kind: AozoraTokenKind<'s>,
    span: Span,
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
        delimited("［＃", command.map(|c| AozoraTokenKind::Command(c)), "］"),
        ruby.map(|r| AozoraTokenKind::Ruby(r)),
        odoriji.map(|o| AozoraTokenKind::Odoriji(o)),
    ))
    .parse_next(input)
}

fn take_until_special<'s>(input: &mut Input<'s>) -> Result<&'s str, ContextError> {
    let end = alt((peek(special).void(), eof.void()));
    repeat_till(0.., any, end)
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
