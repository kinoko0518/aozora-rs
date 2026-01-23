//! aozorahackの青空文庫書式のドキュメントに記載されているところの、前方参照型のパースを行うモジュールです。
//!　ドキュメントは[こちら](https://github.com/aozorahack/specs/blob/master/aozora-text.md#%E5%89%8D%E6%96%B9%E5%8F%82%E7%85%A7%E5%9E%8B%E3%81%A8%E9%96%8B%E5%A7%8B%E7%B5%82%E4%BA%86%E5%9E%8B)
//! から確認できます。

use winnow::{
    Parser,
    combinator::{alt, delimited},
    error::ContextError,
    token::take_until,
};

use crate::{
    prelude::*,
    tokenizer::command::definitions::{bosen, boten},
};

#[derive(Debug, Clone, Copy)]
pub struct BackRefSpec<'s>(pub &'s str);

#[derive(Debug, Clone, Copy)]
pub enum BackRefKind {
    Bold,
    Italic,
    Boten(BotenKind),
    Bosen(BosenKind),
}

#[derive(Debug, Clone, Copy)]
pub struct BackRef<'s> {
    pub kind: BackRefKind,
    pub range: BackRefSpec<'s>,
}

pub fn backref<'s>(input: &mut Input<'s>) -> Result<BackRef<'s>, ContextError> {
    let target = BackRefSpec(delimited("「", take_until(1.., "」"), "」").parse_next(input)?);
    alt((
        "は太字".value(BackRefKind::Bold),
        "は斜体".value(BackRefKind::Italic),
        (
            "に",
            alt((
                bosen.map(|b| BackRefKind::Bosen(b)),
                boten.map(|b| BackRefKind::Boten(b)),
            )),
        )
            .map(|(_, b)| b),
    ))
    .map(|b| BackRef {
        kind: b,
        range: target,
    })
    .parse_next(input)
}
