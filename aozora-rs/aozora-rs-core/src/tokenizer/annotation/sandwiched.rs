#![doc = include_str!("../../../docs/note/sandwiched.md")]

use winnow::{Parser, combinator::alt};

use crate::nihongo::japanese_num;
use crate::tokenizer::{annotation::SandwichedBegin, definition::*};
use crate::*;

/// 行内挟み込み型注記の開始側に対応します。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandwichedBegins {
    /// 「太字」に対応
    BoldBegin,
    /// 「斜体」に対応
    ItalicBegin,
    /// 「傍点」に対応
    BotenBegin(BotenKind),
    /// 「傍線」に対応
    BosenBegin(BosenKind),
    /// 「大見出し」に対応
    AHeadBegin,
    /// 「中見出し」に対応
    BHeadBegin,
    /// 「小見出し」に対応
    CHeadBegin,
    /// 「N段階小さな文字」に対応
    SmallerBegin(usize),
    /// 「N段階大きな文字」に対応
    BiggerBegin(usize),
    /// 「割り注」に対応
    Warichu,
    /// 「横組み」に対応
    HorizontalLayout,
    /// 「行右小書き」に対応
    Sup,
}

impl From<SandwichedBegins> for Deco<'static> {
    fn from(val: SandwichedBegins) -> Self {
        match val {
            SandwichedBegins::BoldBegin => Deco::Bold,
            SandwichedBegins::ItalicBegin => Deco::Italic,
            SandwichedBegins::BosenBegin(b) => Deco::Bosen(b),
            SandwichedBegins::BotenBegin(b) => Deco::Boten(b),
            SandwichedBegins::AHeadBegin => Deco::AHead,
            SandwichedBegins::BHeadBegin => Deco::BHead,
            SandwichedBegins::CHeadBegin => Deco::CHead,
            SandwichedBegins::SmallerBegin(b) => Deco::Smaller(b),
            SandwichedBegins::BiggerBegin(b) => Deco::Bigger(b),
            SandwichedBegins::Warichu => Deco::Warichu,
            SandwichedBegins::HorizontalLayout => Deco::HorizontalLayout,
            SandwichedBegins::Sup => Deco::Sup,
        }
    }
}

impl SandwichedBegin<SandwichedEnds> for SandwichedBegins {
    fn do_match(&self, rhs: &SandwichedEnds) -> bool {
        match self {
            Self::BoldBegin => matches!(rhs, SandwichedEnds::BoldEnd),
            Self::ItalicBegin => matches!(rhs, SandwichedEnds::ItalicEnd),
            Self::BotenBegin(inner) => {
                matches!(rhs, SandwichedEnds::BotenEnd(b) if b == inner)
            }
            Self::BosenBegin(inner) => {
                matches!(rhs, SandwichedEnds::BosenEnd(b) if b == inner)
            }
            Self::AHeadBegin => matches!(rhs, SandwichedEnds::AHeadEnd),
            Self::BHeadBegin => matches!(rhs, SandwichedEnds::BHeadEnd),
            Self::CHeadBegin => matches!(rhs, SandwichedEnds::CHeadEnd),
            Self::SmallerBegin(_) => matches!(rhs, SandwichedEnds::SmallerEnd),
            Self::BiggerBegin(_) => matches!(rhs, SandwichedEnds::BiggerEnd),
            Self::Warichu => matches!(rhs, SandwichedEnds::WarichuEnd),
            Self::HorizontalLayout => matches!(rhs, SandwichedEnds::HorizontalLayout),
            Self::Sup => matches!(rhs, SandwichedEnds::Sup),
        }
    }
}

/// 行内挟み込み型注記の終了側に対応します。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandwichedEnds {
    /// 「太字終わり」に対応
    BoldEnd,
    /// 「斜体終わり」に対応
    ItalicEnd,
    /// 「傍点終わり」に対応
    BotenEnd(BotenKind),
    /// 「傍線終わり」に対応
    BosenEnd(BosenKind),
    /// 「大見出し終わり」に対応
    AHeadEnd,
    /// 「中見出し終わり」に対応
    BHeadEnd,
    /// 「小見出し終わり」に対応
    CHeadEnd,
    /// 「小さな文字終わり」に対応
    SmallerEnd,
    /// 「大きな文字終わり」に対応
    BiggerEnd,
    /// 「割り注終わり」に対応
    WarichuEnd,
    /// 「横組み終わり」に対応
    HorizontalLayout,
    /// 「行右小書き」に対応
    Sup,
}

/// 開始タグと終了タグの直和です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sandwiched {
    /// 開始タグ
    Begin(SandwichedBegins),
    ///　終了タグ
    End(SandwichedEnds),
}

fn sandwiched_begin(input: &mut Input<'_>) -> Result<SandwichedBegins, WinnowError> {
    alt((
        "大見出し".value(SandwichedBegins::AHeadBegin),
        "中見出し".value(SandwichedBegins::BHeadBegin),
        "小見出し".value(SandwichedBegins::CHeadBegin),
        "太字".value(SandwichedBegins::BoldBegin),
        "斜体".value(SandwichedBegins::ItalicBegin),
        boten.map(SandwichedBegins::BotenBegin),
        bosen.map(SandwichedBegins::BosenBegin),
        (japanese_num, "段階小さな文字").map(|(s, _)| SandwichedBegins::SmallerBegin(s)),
        (japanese_num, "段階大きな文字").map(|(b, _)| SandwichedBegins::SmallerBegin(b)),
        "割り注".value(SandwichedBegins::Warichu),
        "横組み".value(SandwichedBegins::HorizontalLayout),
        "行右小書き".value(SandwichedBegins::Sup),
    ))
    .parse_next(input)
}

fn sandwiched_end(input: &mut Input<'_>) -> Result<SandwichedEnds, WinnowError> {
    (
        alt((
            "大見出し".value(SandwichedEnds::AHeadEnd),
            "中見出し".value(SandwichedEnds::BHeadEnd),
            "小見出し".value(SandwichedEnds::CHeadEnd),
            "太字".value(SandwichedEnds::BoldEnd),
            "斜体".value(SandwichedEnds::ItalicEnd),
            boten.map(SandwichedEnds::BotenEnd),
            bosen.map(SandwichedEnds::BosenEnd),
            "小さな文字".value(SandwichedEnds::SmallerEnd),
            "大きな文字".value(SandwichedEnds::BiggerEnd),
            "割り注".value(SandwichedEnds::WarichuEnd),
            "横組み".value(SandwichedEnds::HorizontalLayout),
            "行右小書き".value(SandwichedEnds::Sup),
        )),
        "終わり",
    )
        .map(|(v, _)| v)
        .parse_next(input)
}

pub(crate) fn sandwiched(input: &mut Input<'_>) -> Result<Sandwiched, WinnowError> {
    alt((
        sandwiched_end.map(Sandwiched::End),
        sandwiched_begin.map(Sandwiched::Begin),
    ))
    .parse_next(input)
}
