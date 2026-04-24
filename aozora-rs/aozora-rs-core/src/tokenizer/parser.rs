use winnow::{
    Parser,
    combinator::{alt, delimited, not, peek, repeat},
    token::{any, take_till, take_until},
};

use crate::tokenizer::*;
use crate::{tokenizer::note::command, *};

fn ruby<'s>(input: &mut Input<'s>) -> Result<&'s str, WinnowError> {
    const END: char = '》';
    delimited('《', take_until(1.., END), END).parse_next(input)
}

fn special<'s>(input: &mut Input<'s>) -> Result<AozoraTokenKind<'s>, WinnowError> {
    alt((
        '｜'.value(AozoraTokenKind::RubyDelimiter),
        '\n'.value(AozoraTokenKind::Br),
        delimited(
            "［＃",
            alt((
                command.map(AozoraTokenKind::Note),
                take_until(1.., "］").map(|s| AozoraTokenKind::Note(Note::Unknown(s))),
            )),
            "］",
        ),
        ruby.map(AozoraTokenKind::Ruby),
    ))
    .parse_next(input)
}

fn take_until_special<'s>(input: &mut Input<'s>) -> Result<&'s str, WinnowError> {
    fn fast_skip<'s>(input: &mut Input<'s>) -> Result<(), WinnowError> {
        take_till(1.., |c| matches!(c, '｜' | '\n' | '［' | '《' | '／'))
            .void()
            .parse_next(input)
    }
    fn false_trigger<'s>(input: &mut Input<'s>) -> Result<(), WinnowError> {
        (not(peek(special)), any).void().parse_next(input)
    }

    repeat(1.., alt((fast_skip, false_trigger)))
        .map(|_: ()| ())
        .take()
        .parse_next(input)
}

pub fn tokenize<'s>(input: &mut Input<'s>) -> Result<Vec<Tokenized<'s>>, WinnowError> {
    let mut result: Vec<Tokenized> = repeat(
        0..,
        alt((special, take_until_special.map(AozoraTokenKind::Text)))
            .with_span()
            .map(|(kind, span)| Tokenized { kind, span }),
    )
    .parse_next(input)?;

    result.retain(|token| match &token.kind {
        AozoraTokenKind::Text(t) => !t.is_empty(),
        _ => true,
    });

    Ok(result)
}
