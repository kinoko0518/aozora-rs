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
    Input,
    tokenizer::command::definitions::{BosenKind, BotenKind, bosen, boten},
};

#[derive(Debug, Clone, Copy)]
struct BackRefSpec<'s>(&'s str);

#[derive(Debug, Clone, Copy)]
pub enum BackRefNote<'s> {
    Bold(BackRefSpec<'s>),
    Italic(BackRefSpec<'s>),
    Boten((BackRefSpec<'s>, BotenKind)),
    Bosen((BackRefSpec<'s>, BosenKind)),
}

pub fn backref<'s>(input: &mut Input<'s>) -> Result<BackRefNote<'s>, ContextError> {
    let target = BackRefSpec(delimited("「", take_until(1.., "」"), "」").parse_next(input)?);
    alt((
        "は太字".value(BackRefNote::Bold(target)),
        "は斜体".value(BackRefNote::Italic(target)),
        (
            "に",
            alt((
                bosen.map(|b| BackRefNote::Bosen((target, b))),
                boten.map(|b| BackRefNote::Boten((target, b))),
            )),
        )
            .map(|(_, b)| b),
    ))
    .parse_next(input)
}
