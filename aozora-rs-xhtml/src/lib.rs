use crate::{convert::into_xhtml, dom::into_mapped};
use aozora_rs_core::prelude::*;
use itertools::Itertools;
use winnow::LocatingSlice;

mod convert;
mod definitions;
mod dom;

pub use definitions::*;
pub use dom::{Mapped, MappedToken};

pub struct NovelResult<'s> {
    pub title: &'s str,
    pub author: &'s str,
    pub xhtmls: XHTMLResult<'s>,
    pub errors: Vec<miette::Error>,
}

pub struct NovelResultNoMeta<'s> {
    pub xhtmls: XHTMLResult<'s>,
    pub errors: Vec<miette::Error>,
}

pub struct XHTMLResult<'s> {
    pub xhtmls: Vec<String>,
    pub dependency: Vec<&'s str>,
    pub chapters: Vec<Chapter>,
}

fn from_retokenized<'s>(retokenized: Vec<Retokenized<'s>>) -> XHTMLResult<'s> {
    let mapped = into_mapped(retokenized);
    let dependency = mapped.dependency;
    let xhtmls = mapped
        .xhtmls
        .iter()
        .map(|x| x.iter().map(|m| into_xhtml(m)).join(""))
        .collect::<Vec<String>>();
    let chapters = mapped
        .xhtmls
        .into_iter()
        .flatten()
        .filter_map(|x| x.chapter)
        .collect::<Vec<Chapter>>();
    XHTMLResult {
        xhtmls,
        dependency,
        chapters,
    }
}

/// 一行目はタイトル、二行目は著者名、【テキスト中に現れる記号について】を無視といった
/// 特別な表記を一切考慮せず、入力すべてに対して常に一般的なルールに基づきパースを行います。
///
/// 自身のサイトの中に青空文庫書式で書いたテキストをHTMLとして埋め込みたいときなどにご活用ください。
pub fn convert_with_no_meta<'s>(input: &'s str) -> NovelResultNoMeta<'s> {
    let mut input_slice = LocatingSlice::new(input);
    let tokens = tokenize_nometa(&mut input_slice).unwrap();
    let ((scopenized, flattoken), errors) = scopenize(tokens, input).into_tuple();
    let retokenized = retokenize(flattoken, scopenized);
    let xhtmls = from_retokenized(retokenized);
    NovelResultNoMeta { xhtmls, errors }
}

/// 一行目はタイトル、二行目は著者名、【テキスト中に現れる記号について】を無視といった
/// 特別な表記を考慮し、メタデータとして解析します。
///
/// 既存の青空文庫書式で書かれた作品をパースして独自の表示を行いたいときなどに有用です。
pub fn convert_with_meta<'s>(input: &'s str) -> NovelResult<'s> {
    let ((meta, parsed), errors) = aozora_rs_core::parse(input).into_tuple();

    NovelResult {
        title: meta.title,
        author: meta.author,
        xhtmls: from_retokenized(parsed),
        errors,
    }
}
