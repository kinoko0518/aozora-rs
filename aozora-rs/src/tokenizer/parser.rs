use std::borrow::Cow;

use aozora_rs_gaiji::whole_gaiji_to_char;
use winnow::{
    Parser,
    combinator::{alt, delimited, eof, not, opt, peek, repeat, repeat_till},
    error::ContextError,
    token::{any, take_until},
};

use crate::tokenizer::prelude::*;
use crate::{prelude::*, tokenizer::note::command};

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

fn gaiji_pattern<'s>(input: &mut Input<'s>) -> Result<&'s str, ContextError> {
    ("※", "［＃", take_until(1.., "］"), "］")
        .take()
        .parse_next(input)
}

fn special<'s>(input: &mut Input<'s>) -> Result<AozoraTokenKind<'s>, ContextError> {
    alt((
        '｜'.value(AozoraTokenKind::RubyDelimiter),
        '\n'.value(AozoraTokenKind::Br),
        (
            not('※'),
            delimited(
                "［＃",
                alt((
                    command.map(|c| AozoraTokenKind::Note(c)),
                    take_until(1.., "］").map(|s| AozoraTokenKind::Note(Note::Unknown(s))),
                )),
                "］",
            ),
        )
            .map(|(_, s)| s),
        ruby.map(|r| AozoraTokenKind::Ruby(r)),
        odoriji.map(|o| AozoraTokenKind::Odoriji(o)),
    ))
    .parse_next(input)
}

fn take_until_special<'s>(input: &mut Input<'s>) -> Result<&'s str, ContextError> {
    let end = alt((peek(special).void(), eof.void()));
    repeat_till(1.., alt((gaiji_pattern, any.take())), end)
        .map(|(s, _): ((), _)| s)
        .take()
        .parse_next(input)
}

pub fn tokenize_nometa<'s>(input: &mut Input<'s>) -> Result<Vec<AozoraToken<'s>>, ContextError> {
    let mut result: Vec<AozoraToken> = repeat(
        0..,
        alt((
            special,
            take_until_special
                .map(whole_gaiji_to_char)
                .map(AozoraTokenKind::Text),
        ))
        .with_span()
        .map(|(k, s)| {
            // 外字を扱っていた場合インデクスがずれるため別の計算ロジックを用いる
            let span = if let AozoraTokenKind::Text(Cow::Owned(t)) = &k {
                let length: usize = t.chars().map(|c| c.len_utf8()).sum();
                s.start..(s.start + length)
            } else {
                s
            };
            AozoraToken {
                kind: k,
                span: span,
            }
        }),
    )
    .parse_next(input)?;

    result.retain(|token| match &token.kind {
        AozoraTokenKind::Text(t) => !t.is_empty(),
        _ => true,
    });

    Ok(result)
}
