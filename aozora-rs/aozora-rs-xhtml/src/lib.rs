use aozora_rs_core::*;

mod definitions;
mod xhtmlnize;

pub use definitions::*;

use crate::xhtmlnize::into_xhtml;

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

pub fn retokenized_to_xhtml<'s>(
    retokenized: Vec<Retokenized>,
    meta: AozoraMeta<'s>,
    errors: Vec<miette::Error>,
) -> NovelResult<'s> {
    NovelResult {
        xhtmls: into_xhtml(retokenized),
        meta,
        errors,
    }
}
