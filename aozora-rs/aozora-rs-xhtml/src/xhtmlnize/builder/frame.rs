use std::borrow::Cow;

use crate::xhtmlnize::ast::{BlockNode, ContentNode, HeadingLevel, InlineNode};

pub(crate) enum ContainerFrame<'s> {
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
    pub(crate) fn is_inline_context(&self) -> bool {
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

    pub(crate) fn push_inline(&mut self, inline: InlineNode<'s>) {
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

    pub(crate) fn push_block(&mut self, block: BlockNode<'s>) {
        match self {
            Self::Div { children, .. } => children.push(ContentNode::Block(block)),
            // インラインコンテキストにはブロックは入れられない（型が保証）
            _ => unreachable!("Block elements cannot be pushed into inline context"),
        }
    }
}
