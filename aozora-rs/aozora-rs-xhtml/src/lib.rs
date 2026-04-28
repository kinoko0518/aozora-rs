mod definitions;
mod xhtmlnize;

use aozora_rs_core::Page;
pub use definitions::*;
pub use xhtmlnize::*;

pub struct XHTMLResult {
    pub xhtmls: Vec<String>,
    pub dependency: Vec<String>,
    pub chapters: Vec<Chapter>,
}

/// Vec<Page>からXHTMLResultを生成します。
pub fn retokenized_to_xhtml(pages: Vec<Page<'_>>) -> XHTMLResult {
    let mut converter = XHTMLConverter::new();
    for page in pages {
        converter.feed_page(page);
    }
    converter.convert()
}
