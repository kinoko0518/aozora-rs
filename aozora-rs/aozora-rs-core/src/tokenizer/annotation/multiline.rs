#![doc = include_str!("../../../docs/note/multiline.md")]

use winnow::{
    Parser,
    combinator::{alt, delimited, opt},
};

use crate::{
    WinnowError,
    deco::{BlockIndent, Deco},
    nihongo::japanese_num,
    tokenizer::annotation::{Input, SandwichedBegin},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HangingIndent {
    pub fst_lvl: usize,
    pub snd_lvl: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LowFlying {
    pub level: usize,
}

/// 複数行挟み込み型注記の開始側、［＃ここから……］の形式で表記される側です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultiLineBegins {
    /// ここからN字下げに対応
    BlockIndent(BlockIndent),
    /// ここからN字下げ、折り返してM字下げに対応
    HangingIndent(HangingIndent),
    /// ここから地付きに対応
    Grounded,
    /// ここから地からN字上げ
    LowFlying(LowFlying),
    /// ここからN段階小さな文字
    Smaller(usize),
    /// ここからN段階大きな文字
    Bigger(usize),
    /// ここからN字詰め
    Kerning(usize),
}

impl SandwichedBegin<MultiLineEnds> for MultiLineBegins {
    fn do_match(&self, rhs: &MultiLineEnds) -> bool {
        match self {
            Self::BlockIndent(_) => matches!(rhs, MultiLineEnds::BlockIndentEnd),
            Self::Grounded => matches!(rhs, MultiLineEnds::GroundedEnd),
            Self::HangingIndent(_) => matches!(rhs, MultiLineEnds::BlockIndentEnd),
            Self::LowFlying(_) => matches!(rhs, MultiLineEnds::LowFlyingEnd),
            Self::Smaller(_) => matches!(rhs, MultiLineEnds::SmallEnd),
            Self::Bigger(_) => matches!(rhs, MultiLineEnds::BigEnd),
            Self::Kerning(_) => matches!(rhs, MultiLineEnds::Kerning),
        }
    }
}

impl MultiLineBegins {
    /// 注記を[`Deco`]に変換します。
    pub fn into_deco<'s>(self) -> Deco<'s> {
        match self {
            Self::BlockIndent(b) => Deco::Indent(b.level),
            Self::HangingIndent(h) => Deco::Hanging((h.fst_lvl, h.snd_lvl)),
            Self::Grounded => Deco::Grounded,
            Self::LowFlying(l) => Deco::LowFlying(l.level),
            Self::Smaller(s) => Deco::Smaller(s),
            Self::Bigger(b) => Deco::Bigger(b),
            Self::Kerning(j) => Deco::Kerning(j),
        }
    }
}

/// 複数行挟み込み型注記の終了側、［＃ここで……終わり］の形式で表記される側です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultiLineEnds {
    /// 「ここで字下げ終わり」に対応
    BlockIndentEnd,
    /// 「ここで地付き終わり」
    GroundedEnd,
    /// 「ここで字上げ終わり」に対応
    LowFlyingEnd,
    /// 「ここで小さな文字終わり」に対応
    SmallEnd,
    /// 「ここで大きな文字終わり」に対応
    BigEnd,
    /// 「ここで字詰め終わり」に対応
    Kerning,
}

/// 以下のような形式で記述する複数行挟み込み型の注記に対応します。
/// ```aozorabunko
/// ［＃ここから……］
/// 一行目
/// 二行目
/// ……
/// ［＃ここで……終わり］
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultiLine {
    /// 一部例外と［＃ここから……］のパターンで記述される複数行挟み込み型の開始注記です。
    Begin(MultiLineBegins),
    /// 一部例外と［＃ここで……終わり］のパターンで記述される複数行挟み込み型の終了注記です。
    End(MultiLineEnds),
}

fn jisage<'s>(input: &mut Input<'s>) -> Result<usize, WinnowError> {
    (japanese_num, "字下げ").map(|(u, _)| u).parse_next(input)
}

fn block_indent_begins(input: &mut Input) -> Result<MultiLineBegins, WinnowError> {
    (
        alt((jisage, "改行天付き".value(0))),
        opt(("、折り返して", jisage).map(|(_, o)| o)),
    )
        .map(|(u, o)| match o {
            Some(o) => MultiLineBegins::HangingIndent(HangingIndent {
                fst_lvl: u,
                snd_lvl: o,
            }),
            None => MultiLineBegins::BlockIndent(BlockIndent { level: u }),
        })
        .parse_next(input)
}

fn chitsuki_begins(input: &mut Input) -> Result<MultiLineBegins, WinnowError> {
    "地付き".value(MultiLineBegins::Grounded).parse_next(input)
}

fn chiyose<'s>(input: &mut Input<'s>) -> Result<usize, WinnowError> {
    ("地から", japanese_num, "字上げ")
        .map(|(_, u, _)| u)
        .parse_next(input)
}

fn chiyose_block_begins<'s>(input: &mut Input<'s>) -> Result<MultiLineBegins, WinnowError> {
    chiyose
        .map(|u| MultiLineBegins::LowFlying(LowFlying { level: u }))
        .parse_next(input)
}

fn smaller_block_begins<'s>(input: &mut Input<'s>) -> Result<MultiLineBegins, WinnowError> {
    (japanese_num, "段階小さな文字")
        .map(|(u, _)| MultiLineBegins::Smaller(u))
        .parse_next(input)
}

fn bigger_block_begins<'s>(input: &mut Input<'s>) -> Result<MultiLineBegins, WinnowError> {
    (japanese_num, "段階大きな文字")
        .map(|(u, _)| MultiLineBegins::Bigger(u))
        .parse_next(input)
}

fn kerning_begins<'s>(input: &mut Input<'s>) -> Result<MultiLineBegins, WinnowError> {
    (japanese_num, "字詰め")
        .map(|(u, _)| MultiLineBegins::Kerning(u))
        .parse_next(input)
}

fn multiline_begins<'s>(input: &mut Input<'s>) -> Result<MultiLineBegins, WinnowError> {
    (
        "ここから",
        alt((
            block_indent_begins,
            chitsuki_begins,
            chiyose_block_begins,
            smaller_block_begins,
            bigger_block_begins,
            kerning_begins,
        )),
    )
        .map(|(_, b)| b)
        .parse_next(input)
}

fn multiline_ends<'s>(input: &mut Input<'s>) -> Result<MultiLineEnds, WinnowError> {
    delimited(
        "ここで",
        alt((
            "字下げ".value(MultiLineEnds::BlockIndentEnd),
            "字上げ".value(MultiLineEnds::LowFlyingEnd),
            "地付き".value(MultiLineEnds::GroundedEnd),
            "小さな文字".value(MultiLineEnds::SmallEnd),
            "大きな文字".value(MultiLineEnds::BigEnd),
            "字詰め".value(MultiLineEnds::Kerning),
        )),
        alt(("終わり", "おわり", "終り")),
    )
    .parse_next(input)
}

pub fn multiline<'s>(input: &mut Input<'s>) -> Result<MultiLine, WinnowError> {
    alt((
        multiline_begins.map(MultiLine::Begin),
        multiline_ends.map(MultiLine::End),
    ))
    .parse_next(input)
}
