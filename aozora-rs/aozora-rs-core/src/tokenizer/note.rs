use winnow::{Parser, combinator::alt};

use crate::tokenizer::note::backref::BackRef;
use crate::tokenizer::note::backref::backref;
use crate::tokenizer::note::multiline::multiline;
use crate::tokenizer::note::sandwiched::sandwiched;
use crate::tokenizer::note::single::single;
use crate::tokenizer::note::wholeline::{WholeLine, wholeline};
use crate::tokenizer::*;
use crate::*;

pub mod backref;
#[macro_use]
pub mod multiline;
#[macro_use]
pub mod sandwiched;
pub mod single;
pub mod wholeline;

#[doc = include_str!("../../docs/note/note.md")]
#[derive(Debug, Clone)]
pub enum Note<'s> {
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
    /// いずれのパターンにもマッチしなかった注記が分類されます。
    Unknown(&'s str),
}

/// 開始/終了で構成されるたぐいの注記のためのトレイトです。開始注記に対して実装します。
pub trait SandwichedBegin<E> {
    /// 引数で与えられた終了タグによって自身が閉じられるかを真理値で返却します。
    fn do_match(&self, rhs: &E) -> bool;
}

type RNote<'s> = Result<Note<'s>, WinnowError>;

/// 注記にマッチするパーサーです。
pub fn command<'s>(input: &mut Input<'s>) -> RNote<'s> {
    alt((
        backref.map(Note::BackRef),
        sandwiched.map(Note::Sandwiched),
        multiline.map(Note::Multiline),
        wholeline.map(Note::WholeLine),
        single.map(Note::Single),
    ))
    .parse_next(input)
}
