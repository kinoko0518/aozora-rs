mod deco_begin;
mod deco_end;
mod definitions;
mod validate;

use std::{borrow::Cow, vec};

use aozora_rs_core::{Deco, Page, Retokenized};

use crate::{
    CDepth, Chapter, XHTMLResult,
    xhtmlnize::{
        definitions::{XHTMLKind, XHTMLTag},
        validate::validate_xhtml,
    },
};

use itertools::{Itertools, MultiPeek};
use std::vec::IntoIter;

fn render_xhtml_tags(tags: Vec<XHTMLTag<'_>>) -> String {
    let mut buff = String::new();
    let mut peekable = validate_xhtml(tags).into_iter().peekable();
    let mut indent: usize = 0;

    while let Some(s) = peekable.next() {
        let is_inline = s.kind.is_inline();
        let is_block_end = s.kind.is_block_end();
        let is_block_begin = s.kind.is_block_begin();

        if !is_inline && is_block_end {
            indent = indent.saturating_sub(1);
        }
        buff.extend(std::iter::repeat('\t').take(indent));

        if is_inline {
            // 同行のインライン要素を消化しきる
            buff.push_str(&s.into_htmltag());
            while let Some(inline) = peekable.next_if(|s| s.kind.is_inline()) {
                buff.push_str(&inline.into_htmltag());
            }
            buff.push('\n');
        } else {
            // 非インライン要素を処理
            buff.push_str(&s.into_htmltag());
            buff.push('\n');

            if is_block_begin {
                indent += 1;
            }
        }
    }
    buff
}

pub struct XHTMLConverter<'s> {
    c_depth: CDepth,
    buff: Vec<XHTMLTag<'s>>,
    xhtmls: Vec<Vec<XHTMLTag<'s>>>,
    dependencies: Vec<String>,
    chapters: Vec<Chapter>,
}

impl<'s> XHTMLConverter<'s> {
    pub fn new() -> Self {
        Self {
            c_depth: CDepth::default(),
            buff: Vec::new(),
            xhtmls: Vec::new(),
            dependencies: Vec::new(),
            chapters: Vec::new(),
        }
    }

    fn flush(&mut self) {
        self.xhtmls.push(std::mem::take(&mut self.buff));
    }

    fn parse_chapter<F>(
        &mut self,
        peekable: &mut MultiPeek<IntoIter<Retokenized<'s>>>,
        end_variant: Deco,
        inc_method: F,
    ) -> Chapter
    where
        F: FnOnce(&mut CDepth),
    {
        inc_method(&mut self.c_depth);
        let mut buff = String::new();

        while let Some(s) = peekable.peek() {
            match s {
                Retokenized::DecoEnd(d) if d == &end_variant => break,
                Retokenized::Text(t) => buff.extend(t.chars()),
                _ => (),
            }
        }
        peekable.reset_peek();

        Chapter {
            xhtml_id: self.xhtmls.len(),
            name: buff,
            depth: self.c_depth.clone(),
        }
    }

    /// MultiPeekからトークンを消費して内部状態を更新します。
    pub fn feed(&mut self, peekable: &mut MultiPeek<IntoIter<Retokenized<'s>>>) {
        while let Some(token) = peekable.next() {
            match token {
                Retokenized::Text(t) => {
                    self.buff.push(XHTMLTag::from_kind(XHTMLKind::Text(t)));
                }
                Retokenized::Br => self.buff.push(XHTMLTag::from_kind(XHTMLKind::Br)),
                Retokenized::Kunten(k) => {
                    self.buff.extend([
                        XHTMLTag {
                            kind: XHTMLKind::SupBegin,
                            attributes: vec![Cow::Borrowed("class=\"kunten\"")],
                        },
                        XHTMLTag::from_kind(XHTMLKind::Text(k)),
                        XHTMLTag::from_kind(XHTMLKind::SupEnd),
                    ]);
                }
                Retokenized::Okurigana(o) => {
                    self.buff.extend([
                        XHTMLTag {
                            kind: XHTMLKind::SupBegin,
                            attributes: vec![Cow::Borrowed("class=\"okurigana\"")],
                        },
                        XHTMLTag::from_kind(XHTMLKind::Text(o)),
                        XHTMLTag::from_kind(XHTMLKind::SupEnd),
                    ]);
                }
                Retokenized::Figure(f) => {
                    let size = f
                        .size
                        .map(|size| format!("width=\"{}\" height=\"{}\"", size.0, size.1))
                        .unwrap_or_else(|| "".to_string());
                    self.buff.push(XHTMLTag {
                        kind: XHTMLKind::Img,
                        attributes: vec![
                            Cow::Owned(format!("src=\"{}\"", f.path)),
                            Cow::Owned(size),
                        ],
                    });
                    self.dependencies.push(f.path.to_string());
                }
                Retokenized::DecoBegin(d) => self.handle_deco_begin(peekable, d),
                Retokenized::DecoEnd(e) => self.handle_deco_end(e),
            }
        }
    }

    /// Pageを受け取り、`<div class="page"></div>`で囲んでXHTML変換します。
    pub fn feed_page(&mut self, page: Page<'s>) {
        self.buff.push(XHTMLTag {
            kind: XHTMLKind::DivBegin,
            attributes: vec![Cow::Owned(format!(
                "class=\"{}\"",
                [Some("page"), page.is_centre.then_some("vhcentre")]
                    .into_iter()
                    .filter_map(|s| s)
                    .join(" ")
            ))],
        });

        let mut peekable = itertools::multipeek(page.content);
        self.feed(&mut peekable);

        self.buff.push(XHTMLTag::from_kind(XHTMLKind::DivEnd));
        self.flush();
    }

    pub fn convert(self) -> XHTMLResult {
        XHTMLResult {
            xhtmls: self.xhtmls.into_iter().map(render_xhtml_tags).collect(),
            dependency: self.dependencies,
            chapters: self.chapters,
        }
    }
}
