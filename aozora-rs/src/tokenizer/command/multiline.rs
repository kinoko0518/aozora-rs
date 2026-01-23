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
    tokenizer::command::{Input, SandwichedBegin},
};

#[derive(Debug, Clone, Copy)]
struct BlockIndent {
    level: usize,
}
impl_sandwiched!(MultiLineEnds, BlockIndent, BlockIndentEnd);

#[derive(Debug, Clone, Copy)]
struct HangingIndent {
    fst_lvl: usize,
    snd_lvl: usize,
}
impl_sandwiched!(MultiLineEnds, HangingIndent, BlockIndentEnd);

#[derive(Debug, Clone, Copy)]
struct Grounded;
impl_sandwiched!(MultiLineEnds, Grounded, GroundedEnd);

#[derive(Debug, Clone, Copy)]
struct LowFlying {
    level: usize,
}
impl_sandwiched!(MultiLineEnds, LowFlying, LowFlyingEnd);

#[enum_dispatch]
#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
enum MultiLineEnds {
    BlockIndentEnd,
    GroundedEnd,
    LowFlyingEnd,
}

#[derive(Debug, Clone, Copy)]
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
        .map(|(_, u, o)| match o {
            Some(o) => MultiLineBegins::HangingIndent(HangingIndent {
                fst_lvl: u,
                snd_lvl: o,
            }),
            None => MultiLineBegins::BlockIndent(BlockIndent { level: u }),
        })
        .parse_next(input)
}

fn chitsuki_begins(input: &mut Input) -> Result<MultiLineBegins, ContextError> {
    "ここから地付き"
        .value(MultiLineBegins::Grounded(Grounded))
        .parse_next(input)
}

fn chiyose<'s>(input: &mut Input<'s>) -> Result<usize, ContextError> {
    ("地から", japanese_num, "字上げ")
        .map(|(_, u, _)| u)
        .parse_next(input)
}

fn chiyose_block_begins<'s>(input: &mut Input<'s>) -> Result<MultiLineBegins, ContextError> {
    ("ここから", chiyose)
        .map(|(_, u)| MultiLineBegins::LowFlying(LowFlying { level: u }))
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
    .map(|l| match l {
        "字下げ" => MultiLineEnds::BlockIndentEnd,
        "字寄せ" => MultiLineEnds::LowFlyingEnd,
        "地付け" => MultiLineEnds::GroundedEnd,
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
