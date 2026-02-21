//! aozorahackの青空文庫書式のドキュメントに記載されているところの、開始/終了型のパースを行うモジュールです。
//!　ドキュメントは[こちら](https://github.com/aozorahack/specs/blob/master/aozora-text.md#%E5%89%8D%E6%96%B9%E5%8F%82%E7%85%A7%E5%9E%8B%E3%81%A8%E9%96%8B%E5%A7%8B%E7%B5%82%E4%BA%86%E5%9E%8B)
//! から確認できます。

use winnow::{Parser, combinator::alt, error::ContextError};

use crate::nihongo::japanese_num;
use crate::tokenizer::note::{SandwichedBegin, definitions::*};
use crate::*;

#[derive(Debug, Clone, Copy)]
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
}

impl SandwichedBegins {
    pub fn into_deco(self) -> Deco<'static> {
        match self {
            Self::BoldBegin => Deco::Bold,
            Self::ItalicBegin => Deco::Italic,
            Self::BosenBegin(b) => Deco::Bosen(b.clone()),
            Self::BotenBegin(b) => Deco::Boten(b.clone()),
            Self::AHeadBegin => Deco::AHead,
            Self::BHeadBegin => Deco::BHead,
            Self::CHeadBegin => Deco::CHead,
            Self::SmallerBegin(b) => Deco::Smaller(b),
            Self::BiggerBegin(b) => Deco::Bigger(b),
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
        }
    }
}

#[derive(Debug, Clone, Copy)]
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
}

#[derive(Debug, Clone, Copy)]
pub enum Sandwiched {
    Begin(SandwichedBegins),
    End(SandwichedEnds),
}

fn sandwiched_begin(input: &mut Input<'_>) -> Result<SandwichedBegins, ContextError> {
    alt((
        "大見出し".value(SandwichedBegins::AHeadBegin),
        "中見出し".value(SandwichedBegins::BHeadBegin),
        "小見出し".value(SandwichedBegins::CHeadBegin),
        "太字".value(SandwichedBegins::BoldBegin),
        "斜体".value(SandwichedBegins::ItalicBegin),
        boten.map(|b| SandwichedBegins::BotenBegin(b)),
        bosen.map(|b| SandwichedBegins::BosenBegin(b)),
        (japanese_num, "段階小さな文字").map(|(s, _)| SandwichedBegins::SmallerBegin(s)),
        (japanese_num, "段階大きな文字").map(|(b, _)| SandwichedBegins::SmallerBegin(b)),
    ))
    .parse_next(input)
}

fn sandwiched_end(input: &mut Input<'_>) -> Result<SandwichedEnds, ContextError> {
    alt((
        "大見出し終わり".value(SandwichedEnds::AHeadEnd),
        "中見出し終わり".value(SandwichedEnds::BHeadEnd),
        "小見出し終わり".value(SandwichedEnds::CHeadEnd),
        "太字終わり".value(SandwichedEnds::BoldEnd),
        "斜体終わり".value(SandwichedEnds::ItalicEnd),
        (boten, "終わり").map(|(bt, _)| SandwichedEnds::BotenEnd(bt)),
        (bosen, "終わり").map(|(bs, _)| SandwichedEnds::BosenEnd(bs)),
        "小さな文字終わり".value(SandwichedEnds::SmallerEnd),
        "大きな文字終わり".value(SandwichedEnds::BiggerEnd),
    ))
    .parse_next(input)
}

pub fn sandwiched(input: &mut Input<'_>) -> Result<Sandwiched, ContextError> {
    alt((
        sandwiched_begin.map(|b| Sandwiched::Begin(b)),
        sandwiched_end.map(|e| Sandwiched::End(e)),
    ))
    .parse_next(input)
}
