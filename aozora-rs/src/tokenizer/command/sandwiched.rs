//! aozorahackの青空文庫書式のドキュメントに記載されているところの、開始/終了型のパースを行うモジュールです。
//!　ドキュメントは[こちら](https://github.com/aozorahack/specs/blob/master/aozora-text.md#%E5%89%8D%E6%96%B9%E5%8F%82%E7%85%A7%E5%9E%8B%E3%81%A8%E9%96%8B%E5%A7%8B%E7%B5%82%E4%BA%86%E5%9E%8B)
//! から確認できます。

use winnow::{Parser, combinator::alt, error::ContextError};

use crate::prelude::*;
use crate::tokenizer::command::{SandwichedBegin, definitions::*};

#[derive(Debug, Clone, Copy)]
pub struct Bold;
#[derive(Debug, Clone, Copy)]
pub struct Italic;
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Bosen(BosenKind);
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Boten(BotenKind);

impl_sandwiched!(SandwichedEnds, Bold, BoldEnd);
impl_sandwiched!(SandwichedEnds, Italic, ItalicEnd);

impl_sandwiched_ignore!(SandwichedEnds, Boten, BotenEnd);
impl_sandwiched_ignore!(SandwichedEnds, Bosen, BosenEnd);

#[derive(Debug, Clone, Copy)]
pub enum SandwichedBegins {
    BoldBegin(Bold),
    ItalicBegin(Italic),
    BotenBegin(Boten),
    BosenBegin(Bosen),
}

impl SandwichedBegins {
    pub fn into_deco(self) -> Deco<'static> {
        match self {
            Self::BoldBegin(_) => Deco::Bold,
            Self::ItalicBegin(_) => Deco::Italic,
            Self::BosenBegin(b) => Deco::Bosen(b.0.clone()),
            Self::BotenBegin(b) => Deco::Boten(b.0.clone()),
        }
    }
}

impl SandwichedBegin<SandwichedEnds> for SandwichedBegins {
    fn do_match(&self, rhs: &SandwichedEnds) -> bool {
        match self {
            SandwichedBegins::BoldBegin(inner) => inner.do_match(rhs),
            SandwichedBegins::ItalicBegin(inner) => inner.do_match(rhs),
            SandwichedBegins::BotenBegin(inner) => inner.do_match(rhs),
            SandwichedBegins::BosenBegin(inner) => inner.do_match(rhs),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SandwichedEnds {
    BoldEnd,
    ItalicEnd,
    BotenEnd(BotenKind),
    BosenEnd(BosenKind),
}

#[derive(Debug, Clone, Copy)]
pub enum Sandwiched {
    Begin(SandwichedBegins),
    End(SandwichedEnds),
}

fn sandwiched_begin(input: &mut Input<'_>) -> Result<SandwichedBegins, ContextError> {
    alt((
        "太字".value(SandwichedBegins::BoldBegin(Bold)),
        "斜体".value(SandwichedBegins::ItalicBegin(Italic)),
        boten.map(|b| SandwichedBegins::BotenBegin(Boten(b))),
        bosen.map(|b| SandwichedBegins::BosenBegin(Bosen(b))),
    ))
    .parse_next(input)
}

fn sandwiched_end(input: &mut Input<'_>) -> Result<SandwichedEnds, ContextError> {
    alt((
        "太字終わり".value(SandwichedEnds::BoldEnd),
        "斜体終わり".value(SandwichedEnds::ItalicEnd),
        (boten, "終わり").map(|(bt, _)| SandwichedEnds::BotenEnd(bt)),
        (bosen, "終わり").map(|(bs, _)| SandwichedEnds::BosenEnd(bs)),
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
