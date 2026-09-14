mod deco;
mod frame;

use std::borrow::Cow;
use std::vec::IntoIter;

use aozora_rs_core::Retokenized;
use itertools::MultiPeek;

use crate::{
    CDepth, Chapter,
    xhtmlnize::ast::{BlockNode, InlineNode},
};
use frame::ContainerFrame;

pub struct TreeBuilder<'s> {
    pub(crate) stack: Vec<ContainerFrame<'s>>,
    pub chapters: Vec<Chapter>,
    pub dependencies: Vec<String>,
    pub(crate) c_depth: CDepth,
    pub(crate) xhtml_id: usize,
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

    pub(crate) fn is_in_inline_context(&self) -> bool {
        self.stack.iter().rev().any(|f| f.is_inline_context())
    }

    pub fn push_inline(&mut self, inline: InlineNode<'s>) {
        if let Some(top) = self.stack.last_mut() {
            top.push_inline(inline);
        }
    }

    pub(crate) fn push_frame(&mut self, frame: ContainerFrame<'s>) {
        self.stack.push(frame);
    }

    pub(crate) fn pop_frame(&mut self) -> Option<ContainerFrame<'s>> {
        // ルート要素はpopしない
        if self.stack.len() > 1 {
            self.stack.pop()
        } else {
            None
        }
    }

    pub(crate) fn push_block(&mut self, block: BlockNode<'s>) {
        for frame in self.stack.iter_mut().rev() {
            if let ContainerFrame::Div { children, .. } = frame {
                children.push(crate::xhtmlnize::ast::ContentNode::Block(block));
                return;
            }
        }
    }

    pub(crate) fn close_frame_and_attach(&mut self) {
        if let Some(popped) = self.pop_frame() {
            match popped {
                ContainerFrame::Div {
                    attributes,
                    children,
                } => {
                    self.push_block(BlockNode::Div {
                        attributes,
                        children,
                    });
                }
                ContainerFrame::Paragraph {
                    attributes,
                    children,
                } => {
                    self.push_block(BlockNode::Paragraph {
                        attributes,
                        children,
                    });
                }
                ContainerFrame::Heading {
                    level,
                    attributes,
                    children,
                } => {
                    self.push_block(BlockNode::Heading {
                        level,
                        attributes,
                        children,
                    });
                }
                ContainerFrame::Span {
                    attributes,
                    children,
                } => {
                    if let Some(parent) = self.stack.last_mut() {
                        parent.push_inline(InlineNode::Span {
                            attributes,
                            children,
                        });
                    }
                }
                ContainerFrame::Sup {
                    attributes,
                    children,
                } => {
                    if let Some(parent) = self.stack.last_mut() {
                        parent.push_inline(InlineNode::Sup {
                            attributes,
                            children,
                        });
                    }
                }
                ContainerFrame::Sub {
                    attributes,
                    children,
                } => {
                    if let Some(parent) = self.stack.last_mut() {
                        parent.push_inline(InlineNode::Sub {
                            attributes,
                            children,
                        });
                    }
                }
                ContainerFrame::Ruby { children } => {
                    if let Some(parent) = self.stack.last_mut() {
                        parent.push_inline(InlineNode::Span {
                            attributes: Vec::new(),
                            children,
                        });
                    }
                }
            }
        }
    }

    pub(crate) fn close_ruby(&mut self, rt: Cow<'s, str>) {
        if let Some(popped) = self.pop_frame() {
            if let ContainerFrame::Ruby { children } = popped {
                if let Some(parent) = self.stack.last_mut() {
                    parent.push_inline(InlineNode::Ruby {
                        base: children,
                        rt,
                    });
                }
            } else {
                self.stack.push(popped);
                self.close_frame_and_attach();
            }
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
                let img = InlineNode::Img {
                    attributes: vec![
                        Cow::Owned(format!("src=\"{}\"", f.path)),
                        Cow::Owned(size),
                    ],
                };
                self.push_inline(img);
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
