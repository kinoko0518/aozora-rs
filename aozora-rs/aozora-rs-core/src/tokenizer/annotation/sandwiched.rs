#![doc = include_str!("../../../docs/note/sandwiched.md")]

use winnow::{Parser, combinator::alt};

use crate::nihongo::japanese_num;
use crate::tokenizer::{annotation::SandwichedBegin, definition::*};
use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandwichedBegins {
    BoldBegin,
    ItalicBegin,
    BotenBegin(BotenKind),
    BosenBegin(BosenKind),
    AHeadBegin,
    BHeadBegin,
    CHeadBegin,
    SmallerBegin(usize),
    BiggerBegin(usize),
    Warichu,
    HorizontalLayout,
    Sup,
}

impl Into<Deco<'static>> for SandwichedBegins {
    fn into(self) -> Deco<'static> {
        match self {
            Self::BoldBegin => Deco::Bold,
            Self::ItalicBegin => Deco::Italic,
            Self::BosenBegin(b) => Deco::Bosen(b),
            Self::BotenBegin(b) => Deco::Boten(b),
            Self::AHeadBegin => Deco::AHead,
            Self::BHeadBegin => Deco::BHead,
            Self::CHeadBegin => Deco::CHead,
            Self::SmallerBegin(b) => Deco::Smaller(b),
            Self::BiggerBegin(b) => Deco::Bigger(b),
            Self::Warichu => Deco::Warichu,
            Self::HorizontalLayout => Deco::HorizontalLayout,
            Self::Sup => Deco::Sup,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandwichedEnds {
    BoldEnd,
    ItalicEnd,
    BotenEnd(BotenKind),
    BosenEnd(BosenKind),
    AHeadEnd,
    BHeadEnd,
    CHeadEnd,
    SmallerEnd,
    BiggerEnd,
    WarichuEnd,
    HorizontalLayout,
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
            boten.map(|bt| SandwichedEnds::BotenEnd(bt)),
            bosen.map(|bs| SandwichedEnds::BosenEnd(bs)),
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
