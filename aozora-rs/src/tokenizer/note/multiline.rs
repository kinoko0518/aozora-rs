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

use winnow::{
    Parser,
    combinator::{alt, delimited, opt},
    error::ContextError,
};

use crate::{
    deco::{BlockIndent, Deco},
    nihongo::japanese_num,
    tokenizer::note::{Input, SandwichedBegin},
};

#[derive(Debug, Clone, Copy)]
pub struct HangingIndent {
    fst_lvl: usize,
    snd_lvl: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Grounded;

#[derive(Debug, Clone, Copy)]
pub struct LowFlying {
    level: usize,
}

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

impl SandwichedBegin<MultiLineEnds> for MultiLineBegins {
    fn do_match(&self, rhs: &MultiLineEnds) -> bool {
        match self {
            Self::BlockIndent(_) => matches!(rhs, MultiLineEnds::BlockIndentEnd),
            Self::Grounded(_) => matches!(rhs, MultiLineEnds::GroundedEnd),
            Self::HangingIndent(_) => matches!(rhs, MultiLineEnds::BlockIndentEnd),
            Self::LowFlying(_) => matches!(rhs, MultiLineEnds::LowFlyingEnd),
        }
    }
}

impl MultiLineBegins {
    pub fn into_deco<'s>(self) -> Deco<'s> {
        match self {
            Self::BlockIndent(b) => Deco::Indent(b.level),
            Self::HangingIndent(h) => Deco::Hanging((h.fst_lvl, h.snd_lvl)),
            Self::Grounded(_) => Deco::Grounded,
            Self::LowFlying(l) => Deco::LowFlying(l.level),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MultiLineEnds {
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
    alt((
        multiline_begins.map(|s| MultiLine::Begin(s)),
        multiline_ends.map(|s| MultiLine::End(s)),
    ))
    .parse_next(input)
}
