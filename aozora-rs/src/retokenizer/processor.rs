use std::collections::HashMap;

use miette::Diagnostic;
use thiserror::Error;

use crate::prelude::*;

enum SandwichedDeco {
    Bold,
    Italic,
    Bosen(BosenKind),
    Boten(BotenKind),
    Indent(usize),
    Hanging((usize, usize)),
    Grounded,
    LowFlying(usize),
    Mama,
    Small(usize),
    Big(usize),
}

pub struct RubiedText<'s> {
    text: &'s str,
    ruby: &'s str,
}

pub enum Retokenized<'s> {
    RubiedText(RubiedText<'s>),
    PlainText(&'s str),
    AHead(&'s str),
    BHead(&'s str),
    CHead(&'s str),
    HinV(&'s str),
    Odoriji(Odoriji),
    Break(Break),
    Figure(Figure<'s>),
}

enum Element<'s> {
    DecoBegin(SandwichedDeco),
    DecoEnd(SandwichedDeco),
    Flat(Retokenized<'s>),
}

#[derive(Error, Debug, Diagnostic)]
#[error("入力が空です")]
#[diagnostic(
    code(aozora_rs::input_is_empty_on_retokenize),
    url(docsrs),
    help("このエラーは再トークン化層で発生しています。")
)]
pub struct EmptyInput;

pub fn retokenize<'s>(
    mut flat: Vec<(FlatToken, Span)>,
    mut deco: HashMap<usize, Scope>,
) -> Result<Vec<Element<'s>>, miette::Error> {
    let last = flat
        .last()
        .map(|(_, span)| span.end)
        .ok_or(EmptyInput.into())?;
    while let Some((token, span)) = flat.pop() {
        match token {}
    }
    Ok(())
}
