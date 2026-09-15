pub mod ast;
pub mod builder;

use aozora_rs_core::Page;
use crate::{CDepth, Chapter, XHTMLResult};
use builder::TreeBuilder;

pub struct XHTMLConverter<'s> {
    c_depth: CDepth,
    xhtmls: Vec<String>,
    dependencies: Vec<String>,
    chapters: Vec<Chapter>,
    _marker: std::marker::PhantomData<&'s ()>,
}

impl<'s> XHTMLConverter<'s> {
    pub fn new() -> Self {
        Self {
            c_depth: CDepth::default(),
            xhtmls: Vec::new(),
            dependencies: Vec::new(),
            chapters: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }

    /// Pageを受け取り、型付きASTツリーを構築してXHTML文字列を生成します。
    pub fn feed_page(&mut self, page: Page<'s>) {
        let mut builder = TreeBuilder::new(
            page.is_centre,
            self.xhtmls.len(),
            self.c_depth.clone(),
        );

        let mut peekable = itertools::multipeek(page.content);
        while let Some(token) = peekable.next() {
            builder.feed_token(token, &mut peekable);
        }

        let (tree, chapters, deps, new_depth) = builder.finish();
        self.c_depth = new_depth;
        self.chapters.extend(chapters);
        self.dependencies.extend(deps);

        let mut rendered = String::new();
        tree.render(&mut rendered, 0);
        self.xhtmls.push(rendered);
    }

    pub fn convert(self) -> XHTMLResult {
        XHTMLResult {
            xhtmls: self.xhtmls,
            dependency: self.dependencies,
            chapters: self.chapters,
        }
    }
}

impl Default for XHTMLConverter<'_> {
    fn default() -> Self {
        Self::new()
    }
}
