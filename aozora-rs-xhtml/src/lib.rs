use crate::{
    convert::{XHTMLContext, into_xhtml},
    dom::into_mapped,
};
use aozora_rs_core::*;
use std::fmt::Write;

mod convert;
mod definitions;
mod dom;

pub use definitions::*;
pub use dom::{Mapped, MappedToken};

pub struct NovelResult<'s> {
    pub xhtmls: XHTMLResult,
    pub meta: AozoraMeta<'s>,
    pub errors: Vec<miette::Report>,
}

pub struct XHTMLResult {
    pub xhtmls: Vec<String>,
    pub dependency: Vec<String>,
    pub chapters: Vec<Chapter>,
}

fn from_retokenized<'s>(retokenized: Vec<Retokenized<'s>>) -> XHTMLResult {
    let mapped = into_mapped(retokenized);
    let dependency = mapped
        .dependency
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let xhtmls = mapped
        .xhtmls
        .iter()
        .map(|x| {
            let mut acc = String::new();
            let mut context = XHTMLContext::default();
            let mut peekable = x.iter().peekable();

            while let Some(token) = peekable.next() {
                writeln!(acc, "{}", into_xhtml(token, peekable.peek(), &mut context)).unwrap();
            }
            acc
        })
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

pub fn retokenized_to_xhtml<'s>(
    retokenized: Vec<Retokenized>,
    meta: AozoraMeta<'s>,
    errors: Vec<miette::Error>,
) -> NovelResult<'s> {
    NovelResult {
        xhtmls: from_retokenized(retokenized),
        meta,
        errors,
    }
}
