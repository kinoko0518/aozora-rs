//! 独自の分類に基づく、複数行を以下のようなスタイルで囲む種類の注記をパースします。
//! [ブロックでの字下げ](https://www.aozora.gr.jp/annotation/layout_2.html#jisage)
//! などが該当します。
//!
//! ```
//! ［＃注記］
//! 一行目
//! 二行目
//! 三行目…
//! ［＃注記閉じ］
//! ```

use enum_dispatch::enum_dispatch;
use winnow::{
    Parser,
    combinator::{alt, delimited, opt},
    error::ContextError,
};

use crate::{
    nihongo::japanese_num,
    tokenizer::{
        Span,
        command::{Input, SandwichedBegin},
    },
};

struct BlockIndent {
    span: Span,
    level: usize,
}
impl_sandwiched!(MultiLineEnds, BlockIndent, BlockIndentEnd);

struct HangingIndent {
    span: Span,
    fst_lvl: usize,
    snd_lvl: usize,
}
impl_sandwiched!(MultiLineEnds, HangingIndent, BlockIndentEnd);

struct Grounded {
    span: Span,
}
impl_sandwiched!(MultiLineEnds, Grounded, GroundedEnd);

struct LowFlying {
    span: Span,
    level: usize,
}
impl_sandwiched!(MultiLineEnds, LowFlying, LowFlyingEnd);

#[enum_dispatch]
pub enum MultiLineBegins {
    /// 参照：https://www.aozora.gr.jp/annotation/layout_2.html#jisage
    BlockIndent(BlockIndent),
    /// 参照：https://www.aozora.gr.jp/annotation/layout_2.html#ototsu
    HangingIndent(HangingIndent),
    /// 参照：https://www.aozora.gr.jp/annotation/layout_2.html#chitsuki
    Grounded(Grounded),
    /// 参照：https://www.aozora.gr.jp/annotation/layout_2.html#chiyose
    LowFlying(LowFlying),
}

enum MultiLineEnds {
    BlockIndentEnd(Span),
    GroundedEnd(Span),
    LowFlyingEnd(Span),
}

pub enum MultiLine {
    Begin(MultiLineBegins),
    End(MultiLineEnds),
}

fn jisage<'s>(input: &mut Input<'s>) -> Result<usize, ContextError> {
    (japanese_num, "字下げ").map(|(u, _)| u).parse_next(input)
}

fn block_indent_begins(input: &mut Input) -> Result<MultiLineBegins, ContextError> {
    (
        "ここから",
        alt((jisage, "改行天付き".value(0))),
        opt(("、折り返して", jisage).map(|(_, o)| o)),
    )
        .with_span()
        .map(|((_, u, o), s)| match o {
            Some(o) => MultiLineBegins::HangingIndent(HangingIndent {
                span: s,
                fst_lvl: u,
                snd_lvl: o,
            }),
            None => MultiLineBegins::BlockIndent(BlockIndent { span: s, level: u }),
        })
        .parse_next(input)
}

fn chitsuki_begins(input: &mut Input) -> Result<MultiLineBegins, ContextError> {
    "ここから地付き"
        .with_span()
        .map(|(_, s)| MultiLineBegins::Grounded(Grounded { span: s }))
        .parse_next(input)
}

fn chiyose<'s>(input: &mut Input<'s>) -> Result<usize, ContextError> {
    ("地から", japanese_num, "字上げ")
        .map(|(_, u, _)| u)
        .parse_next(input)
}

fn chiyose_block_begins<'s>(input: &mut Input<'s>) -> Result<MultiLineBegins, ContextError> {
    ("ここから", chiyose)
        .with_span()
        .map(|((_, u), s)| MultiLineBegins::LowFlying(LowFlying { span: s, level: u }))
        .parse_next(input)
}

fn multiline_begins<'s>(input: &mut Input<'s>) -> Result<MultiLineBegins, ContextError> {
    alt((block_indent_begins, chitsuki_begins, chiyose_block_begins)).parse_next(input)
}

fn multiline_ends<'s>(input: &mut Input<'s>) -> Result<MultiLineEnds, ContextError> {
    delimited(
        "ここで",
        alt(("字下げ", "字寄せ", "地付け")),
        alt(("終わり", "おわり")),
    )
    .with_span()
    .map(|(l, s)| match l {
        "字下げ" => MultiLineEnds::BlockIndentEnd(s),
        "字寄せ" => MultiLineEnds::LowFlyingEnd(s),
        "地付け" => MultiLineEnds::GroundedEnd(s),
        _ => unreachable!(),
    })
    .parse_next(input)
}

pub fn multiline<'s>(input: &mut Input<'s>) -> Result<MultiLine, ContextError> {
    (
        alt((
            multiline_begins.map(|s| MultiLine::Begin(s)),
            multiline_ends.map(|s| MultiLine::End(s)),
        )),
        '\n',
    )
        .map(|(u, _)| u)
        .parse_next(input)
}
