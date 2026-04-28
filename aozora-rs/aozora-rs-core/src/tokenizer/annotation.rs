use winnow::{Parser, combinator::alt};

use crate::tokenizer::annotation::{
    backref::{BackRef, backref},
    multiline::multiline,
    pagedef::pagedef,
    sandwiched::sandwiched,
    single::single,
    wholeline::{WholeLine, wholeline},
};
use crate::*;

pub mod backref;
#[macro_use]
pub mod multiline;
#[macro_use]
pub mod sandwiched;
pub mod pagedef;
pub mod single;
pub mod wholeline;

#[doc = include_str!("../../docs/note/note.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Annotation<'s> {
    #[doc = "../../docs/note/backref.md"]
    BackRef(BackRef<'s>),
    #[doc = "../../docs/note/sandwitched.md"]
    Sandwiched(Sandwiched),
    #[doc = "../../docs/note/multiline.md"]
    Multiline(MultiLine),
    #[doc = "../../docs/note/single.md"]
    Single(Single<'s>),
    #[doc = "../../docs/note/wholeline.md"]
    WholeLine(WholeLine),
    /// ページ全体を定義する注記が分類されます。
    PageDef(PageDef),
    /// いずれのパターンにもマッチしなかった注記が分類されます。
    Unknown(&'s str),
}

/// 開始/終了で構成されるたぐいの注記のためのトレイトです。開始注記に対して実装します。
pub trait SandwichedBegin<E> {
    /// 引数で与えられた終了タグによって自身が閉じられるかを真理値で返却します。
    fn do_match(&self, rhs: &E) -> bool;
}

type RNote<'s> = Result<Annotation<'s>, WinnowError>;

/// 注記にマッチするパーサーです。
pub fn command<'s>(input: &mut Input<'s>) -> RNote<'s> {
    alt((
        backref.map(Annotation::BackRef),
        sandwiched.map(Annotation::Sandwiched),
        multiline.map(Annotation::Multiline),
        wholeline.map(Annotation::WholeLine),
        single.map(Annotation::Single),
        pagedef.map(Annotation::PageDef),
    ))
    .parse_next(input)
}
