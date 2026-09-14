use std::borrow::Cow;
use std::vec::IntoIter;

use aozora_rs_core::{BosenKind, BotenKind, Deco, Retokenized};
use itertools::MultiPeek;

use crate::{
    CDepth, Chapter,
    xhtmlnize::ast::{BlockNode, ContentNode, HeadingLevel, InlineNode},
};

enum ContainerFrame<'s> {
    Div {
        attributes: Vec<Cow<'s, str>>,
        children: Vec<ContentNode<'s>>,
    },
    Paragraph {
        attributes: Vec<Cow<'s, str>>,
        children: Vec<InlineNode<'s>>,
    },
    Heading {
        level: HeadingLevel,
        attributes: Vec<Cow<'s, str>>,
        children: Vec<InlineNode<'s>>,
    },
    Span {
        attributes: Vec<Cow<'s, str>>,
        children: Vec<InlineNode<'s>>,
    },
    Ruby {
        children: Vec<InlineNode<'s>>,
    },
    Sup {
        attributes: Vec<Cow<'s, str>>,
        children: Vec<InlineNode<'s>>,
    },
    Sub {
        attributes: Vec<Cow<'s, str>>,
        children: Vec<InlineNode<'s>>,
    },
}

impl<'s> ContainerFrame<'s> {
    fn is_inline_context(&self) -> bool {
        matches!(
            self,
            Self::Paragraph { .. }
                | Self::Heading { .. }
                | Self::Span { .. }
                | Self::Ruby { .. }
                | Self::Sup { .. }
                | Self::Sub { .. }
        )
    }

    fn push_inline(&mut self, inline: InlineNode<'s>) {
        match self {
            Self::Div { children, .. } => children.push(ContentNode::Inline(inline)),
            Self::Paragraph { children, .. } => children.push(inline),
            Self::Heading { children, .. } => children.push(inline),
            Self::Span { children, .. } => children.push(inline),
            Self::Ruby { children } => children.push(inline),
            Self::Sup { children, .. } => children.push(inline),
            Self::Sub { children, .. } => children.push(inline),
        }
    }

    fn push_block(&mut self, block: BlockNode<'s>) {
        match self {
            Self::Div { children, .. } => children.push(ContentNode::Block(block)),
            // インラインコンテキストにはブロックは入れられない（型が保証）
            _ => unreachable!("Block elements cannot be pushed into inline context"),
        }
    }
}

pub struct TreeBuilder<'s> {
    stack: Vec<ContainerFrame<'s>>,
    pub chapters: Vec<Chapter>,
    pub dependencies: Vec<String>,
    c_depth: CDepth,
    xhtml_id: usize,
}

impl<'s> TreeBuilder<'s> {
    pub fn new(is_centre: bool, xhtml_id: usize, c_depth: CDepth) -> Self {
        let class = if is_centre {
            "class=\"page vhcentre\""
        } else {
            "class=\"page\""
        };
        let root = ContainerFrame::Div {
            attributes: vec![Cow::Borrowed(class)],
            children: Vec::new(),
        };

        Self {
            stack: vec![root],
            chapters: Vec::new(),
            dependencies: Vec::new(),
            c_depth,
            xhtml_id,
        }
    }

    pub fn get_cdepth(&self) -> CDepth {
        self.c_depth.clone()
    }

    fn is_in_inline_context(&self) -> bool {
        self.stack.iter().rev().any(|f| f.is_inline_context())
    }

    pub fn push_inline(&mut self, inline: InlineNode<'s>) {
        if let Some(top) = self.stack.last_mut() {
            top.push_inline(inline);
        }
    }

    fn push_frame(&mut self, frame: ContainerFrame<'s>) {
        self.stack.push(frame);
    }

    fn pop_frame(&mut self) -> Option<ContainerFrame<'s>> {
        // ルート要素はpopしない
        if self.stack.len() > 1 {
            self.stack.pop()
        } else {
            None
        }
    }

    fn close_frame_and_attach(&mut self) {
        if let Some(popped) = self.pop_frame() {
            if let Some(parent) = self.stack.last_mut() {
                match popped {
                    ContainerFrame::Div {
                        attributes,
                        children,
                    } => {
                        parent.push_block(BlockNode::Div {
                            attributes,
                            children,
                        });
                    }
                    ContainerFrame::Paragraph {
                        attributes,
                        children,
                    } => {
                        parent.push_block(BlockNode::Paragraph {
                            attributes,
                            children,
                        });
                    }
                    ContainerFrame::Heading {
                        level,
                        attributes,
                        children,
                    } => {
                        parent.push_block(BlockNode::Heading {
                            level,
                            attributes,
                            children,
                        });
                    }
                    ContainerFrame::Span {
                        attributes,
                        children,
                    } => {
                        parent.push_inline(InlineNode::Span {
                            attributes,
                            children,
                        });
                    }
                    ContainerFrame::Sup {
                        attributes,
                        children,
                    } => {
                        parent.push_inline(InlineNode::Sup {
                            attributes,
                            children,
                        });
                    }
                    ContainerFrame::Sub {
                        attributes,
                        children,
                    } => {
                        parent.push_inline(InlineNode::Sub {
                            attributes,
                            children,
                        });
                    }
                    ContainerFrame::Ruby { children } => {
                        // ルビはrtが必要なので、明示的なclose_rubyで処理される
                        parent.push_inline(InlineNode::Span {
                            attributes: Vec::new(),
                            children,
                        });
                    }
                }
            }
        }
    }

    fn close_ruby(&mut self, rt: Cow<'s, str>) {
        if let Some(popped) = self.pop_frame() {
            if let ContainerFrame::Ruby { children } = popped {
                if let Some(parent) = self.stack.last_mut() {
                    parent.push_inline(InlineNode::Ruby {
                        base: children,
                        rt,
                    });
                }
            } else {
                // ルビ以外のフレームだった場合は元に戻してattach
                self.stack.push(popped);
                self.close_frame_and_attach();
            }
        }
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
            xhtml_id: self.xhtml_id,
            name: buff,
            depth: self.c_depth.clone(),
        }
    }

    pub fn handle_deco_begin(
        &mut self,
        peekable: &mut MultiPeek<IntoIter<Retokenized<'s>>>,
        d: Deco<'s>,
    ) {
        match d {
            Deco::AHead => {
                let chapter = self.parse_chapter(peekable, Deco::AHead, |c| c.increament_a());
                self.push_frame(ContainerFrame::Heading {
                    level: HeadingLevel::H1,
                    attributes: vec![
                        Cow::Borrowed("class=\"a_head\""),
                        Cow::Owned(format!("id=\"{}\"", chapter.get_id())),
                    ],
                    children: Vec::new(),
                });
                self.chapters.push(chapter);
            }
            Deco::BHead => {
                let chapter = self.parse_chapter(peekable, Deco::BHead, |c| c.increament_b());
                self.push_frame(ContainerFrame::Heading {
                    level: HeadingLevel::H2,
                    attributes: vec![
                        Cow::Borrowed("class=\"b_head\""),
                        Cow::Owned(format!("id=\"{}\"", chapter.get_id())),
                    ],
                    children: Vec::new(),
                });
                self.chapters.push(chapter);
            }
            Deco::CHead => {
                let chapter = self.parse_chapter(peekable, Deco::CHead, |c| c.increament_c());
                self.push_frame(ContainerFrame::Heading {
                    level: HeadingLevel::H3,
                    attributes: vec![
                        Cow::Borrowed("class=\"c_head\""),
                        Cow::Owned(format!("id=\"{}\"", chapter.get_id())),
                    ],
                    children: Vec::new(),
                });
                self.chapters.push(chapter);
            }
            Deco::Bold => {
                self.push_frame(ContainerFrame::Span {
                    attributes: vec![Cow::Borrowed("class=\"bold\"")],
                    children: Vec::new(),
                });
            }
            Deco::Italic => {
                self.push_frame(ContainerFrame::Span {
                    attributes: vec![Cow::Borrowed("class=\"italic\"")],
                    children: Vec::new(),
                });
            }
            Deco::Ruby(_) | Deco::Mama => {
                self.push_frame(ContainerFrame::Ruby {
                    children: Vec::new(),
                });
            }
            Deco::Bosen(b) => {
                self.push_frame(ContainerFrame::Span {
                    attributes: match b {
                        BosenKind::Chain => vec![Cow::Borrowed("class=\"bosen-chain\"")],
                        BosenKind::Plain => vec![Cow::Borrowed("class=\"bosen-solid\"")],
                        BosenKind::Double => vec![Cow::Borrowed("class=\"bosen-double\"")],
                        BosenKind::Dashed => vec![Cow::Borrowed("class=\"bosen-dashed\"")],
                        BosenKind::Wavy => vec![Cow::Borrowed("class=\"bosen-wavy\"")],
                    },
                    children: Vec::new(),
                });
            }
            Deco::Boten(b) => {
                self.push_frame(ContainerFrame::Span {
                    attributes: match b {
                        BotenKind::Circle => vec![Cow::Borrowed("class=\"circle\"")],
                        BotenKind::CircleFilled => vec![Cow::Borrowed("class=\"circle-filled\"")],
                        BotenKind::Sesame => vec![Cow::Borrowed("class=\"sesame\"")],
                        BotenKind::DoubleCircle => vec![Cow::Borrowed("class=\"double-circle\"")],
                        BotenKind::Hebinome => vec![Cow::Borrowed("class=\"hebinome\"")],
                        BotenKind::Triangle => vec![Cow::Borrowed("class=\"triangle\"")],
                        BotenKind::TriangleFilled => {
                            vec![Cow::Borrowed("class=\"triangle-filled\"")]
                        }
                        BotenKind::Crossing => vec![Cow::Borrowed("class=\"crossing\"")],
                    },
                    children: Vec::new(),
                });
            }
            Deco::Indent(i) => {
                let attr = vec![Cow::Owned(format!("style=\"padding-inline-start: {}em;\"", i))];
                if self.is_in_inline_context() {
                    self.push_frame(ContainerFrame::Span {
                        attributes: attr,
                        children: Vec::new(),
                    });
                } else {
                    self.push_frame(ContainerFrame::Div {
                        attributes: attr,
                        children: Vec::new(),
                    });
                }
            }
            Deco::Hanging((h, j)) => {
                let attr = vec![Cow::Owned(format!(
                    "style=\"padding-inline-start: {}em; text-indent: {}em;\"",
                    j,
                    (h as i32) - (j as i32)
                ))];
                if self.is_in_inline_context() {
                    self.push_frame(ContainerFrame::Span {
                        attributes: attr,
                        children: Vec::new(),
                    });
                } else {
                    self.push_frame(ContainerFrame::Div {
                        attributes: attr,
                        children: Vec::new(),
                    });
                }
            }
            Deco::Grounded => {
                self.push_frame(ContainerFrame::Paragraph {
                    attributes: vec![Cow::Borrowed("class=\"grounded\"")],
                    children: Vec::new(),
                });
            }
            Deco::LowFlying(l) => {
                self.push_frame(ContainerFrame::Paragraph {
                    attributes: vec![Cow::Owned(format!(
                        "style=\"text-align: right; padding-inline-end: {}em;\"",
                        l
                    ))],
                    children: Vec::new(),
                });
            }
            Deco::HinV => {
                self.push_frame(ContainerFrame::Span {
                    attributes: vec![Cow::Borrowed("class=\"hinv\"")],
                    children: Vec::new(),
                });
            }
            Deco::Bigger(b) => {
                self.push_frame(ContainerFrame::Span {
                    attributes: vec![Cow::Borrowed(match b {
                        1 => "style=\"font-size: large\"",
                        2 => "style=\"font-size: x-large\"",
                        _ => "style=\"font-size: xx-large\"",
                    })],
                    children: Vec::new(),
                });
            }
            Deco::Smaller(b) => {
                self.push_frame(ContainerFrame::Span {
                    attributes: vec![Cow::Borrowed(match b {
                        1 => "style=\"font-size: small\"",
                        2 => "style=\"font-size: x-small\"",
                        _ => "style=\"font-size: xx-small\"",
                    })],
                    children: Vec::new(),
                });
            }
            Deco::VHCentre => {
                let attr = vec![Cow::Borrowed("class=\"vhcentre\"")];
                if self.is_in_inline_context() {
                    self.push_frame(ContainerFrame::Span {
                        attributes: attr,
                        children: Vec::new(),
                    });
                } else {
                    self.push_frame(ContainerFrame::Div {
                        attributes: attr,
                        children: Vec::new(),
                    });
                }
            }
            Deco::Warichu => {
                self.push_frame(ContainerFrame::Span {
                    attributes: vec![Cow::Borrowed("class=\"warichu\"")],
                    children: Vec::new(),
                });
            }
            Deco::HorizontalLayout => {
                let attr = vec![Cow::Borrowed("class=\"horizontal-block\"")];
                if self.is_in_inline_context() {
                    self.push_frame(ContainerFrame::Span {
                        attributes: attr,
                        children: Vec::new(),
                    });
                } else {
                    self.push_frame(ContainerFrame::Div {
                        attributes: attr,
                        children: Vec::new(),
                    });
                }
            }
            Deco::Kerning(k) => {
                let attr = vec![Cow::Owned(format!("style=\"max-inline-size: {}em;\"", k))];
                if self.is_in_inline_context() {
                    self.push_frame(ContainerFrame::Span {
                        attributes: attr,
                        children: Vec::new(),
                    });
                } else {
                    self.push_frame(ContainerFrame::Div {
                        attributes: attr,
                        children: Vec::new(),
                    });
                }
            }
            Deco::Sub => {
                self.push_frame(ContainerFrame::Sub {
                    attributes: Vec::new(),
                    children: Vec::new(),
                });
            }
            Deco::Sup => {
                self.push_frame(ContainerFrame::Sup {
                    attributes: Vec::new(),
                    children: Vec::new(),
                });
            }
        }
    }

    pub fn handle_deco_end(&mut self, e: Deco<'s>) {
        match e {
            Deco::Ruby(r) => self.close_ruby(Cow::Borrowed(r)),
            Deco::Mama => self.close_ruby(Cow::Borrowed("ママ")),
            _ => self.close_frame_and_attach(),
        }
    }

    pub fn feed_token(
        &mut self,
        token: Retokenized<'s>,
        peekable: &mut MultiPeek<IntoIter<Retokenized<'s>>>,
    ) {
        match token {
            Retokenized::Text(t) => {
                self.push_inline(InlineNode::Text(Cow::Borrowed(t)));
            }
            Retokenized::Br => {
                self.push_inline(InlineNode::Br);
            }
            Retokenized::Kunten(k) => {
                self.push_inline(InlineNode::Sup {
                    attributes: vec![Cow::Borrowed("class=\"kunten\"")],
                    children: vec![InlineNode::Text(Cow::Borrowed(k))],
                });
            }
            Retokenized::Okurigana(o) => {
                self.push_inline(InlineNode::Sup {
                    attributes: vec![Cow::Borrowed("class=\"okurigana\"")],
                    children: vec![InlineNode::Text(Cow::Borrowed(o))],
                });
            }
            Retokenized::Figure(f) => {
                let size = f
                    .size
                    .map(|size| format!("width=\"{}\" height=\"{}\"", size.0, size.1))
                    .unwrap_or_else(|| "".to_string());
                let img = BlockNode::Img {
                    attributes: vec![
                        Cow::Owned(format!("src=\"{}\"", f.path)),
                        Cow::Owned(size),
                    ],
                };
                if let Some(top) = self.stack.last_mut() {
                    top.push_block(img);
                }
                self.dependencies.push(f.path.to_string());
            }
            Retokenized::DecoBegin(d) => self.handle_deco_begin(peekable, d),
            Retokenized::DecoEnd(e) => self.handle_deco_end(e),
        }
    }

    /// すべての未終了フレームを閉じて、ルートのBlockNode::Divを完成させる
    pub fn finish(mut self) -> (BlockNode<'s>, Vec<Chapter>, Vec<String>, CDepth) {
        while self.stack.len() > 1 {
            self.close_frame_and_attach();
        }

        let root = match self.stack.pop().unwrap() {
            ContainerFrame::Div {
                attributes,
                children,
            } => BlockNode::Div {
                attributes,
                children,
            },
            _ => unreachable!("Root frame must be Div"),
        };

        (root, self.chapters, self.dependencies, self.c_depth)
    }
}
