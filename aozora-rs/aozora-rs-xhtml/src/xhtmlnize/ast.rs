use std::borrow::Cow;
use std::fmt::Write;

/// 見出しレベル
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadingLevel {
    H1,
    H2,
    H3,
}

impl HeadingLevel {
    pub fn tag_name(&self) -> &'static str {
        match self {
            Self::H1 => "h1",
            Self::H2 => "h2",
            Self::H3 => "h3",
        }
    }
}

/// インライン要素（見出し、段落、スパンの中にのみ存在可能）
#[derive(Debug, Clone, PartialEq)]
pub enum InlineNode<'s> {
    Text(Cow<'s, str>),
    Br,
    Img {
        attributes: Vec<Cow<'s, str>>,
    },
    Span {
        attributes: Vec<Cow<'s, str>>,
        children: Vec<InlineNode<'s>>,
    },
    Ruby {
        base: Vec<InlineNode<'s>>,
        rt: Cow<'s, str>,
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

impl<'s> InlineNode<'s> {
    pub fn render(&self, buff: &mut String) {
        match self {
            InlineNode::Text(t) => {
                buff.push_str(&quick_xml::escape::escape(&**t));
            }
            InlineNode::Br => {
                buff.push_str("<br />");
            }
            InlineNode::Img { attributes } => {
                buff.push_str("<img");
                for attr in attributes {
                    if !attr.is_empty() {
                        write!(buff, " {attr}").unwrap();
                    }
                }
                buff.push_str(" />");
            }
            InlineNode::Span { attributes, children } => {
                buff.push_str("<span");
                for attr in attributes {
                    write!(buff, " {attr}").unwrap();
                }
                buff.push('>');
                for child in children {
                    child.render(buff);
                }
                buff.push_str("</span>");
            }
            InlineNode::Ruby { base, rt } => {
                buff.push_str("<ruby>");
                for b in base {
                    b.render(buff);
                }
                buff.push_str("<rt>");
                buff.push_str(&quick_xml::escape::escape(&**rt));
                buff.push_str("</rt></ruby>");
            }
            InlineNode::Sup { attributes, children } => {
                buff.push_str("<sup");
                for attr in attributes {
                    write!(buff, " {attr}").unwrap();
                }
                buff.push('>');
                for child in children {
                    child.render(buff);
                }
                buff.push_str("</sup>");
            }
            InlineNode::Sub { attributes, children } => {
                buff.push_str("<sub");
                for attr in attributes {
                    write!(buff, " {attr}").unwrap();
                }
                buff.push('>');
                for child in children {
                    child.render(buff);
                }
                buff.push_str("</sub>");
            }
        }
    }
}

/// ブロック要素
#[derive(Debug, Clone, PartialEq)]
pub enum BlockNode<'s> {
    /// 段落（地付きなどの特殊段落を含む）
    Paragraph {
        attributes: Vec<Cow<'s, str>>,
        children: Vec<InlineNode<'s>>,
    },
    /// 見出し。重要な不変条件：子要素はインラインのみ（ブロック要素の混入が型レベルで不可能）
    Heading {
        level: HeadingLevel,
        attributes: Vec<Cow<'s, str>>,
        children: Vec<InlineNode<'s>>,
    },
    /// div コンテナ
    Div {
        attributes: Vec<Cow<'s, str>>,
        children: Vec<ContentNode<'s>>,
    },
    /// 画像
    Img {
        attributes: Vec<Cow<'s, str>>,
    },
}

impl<'s> BlockNode<'s> {
    pub fn render(&self, buff: &mut String, indent: usize) {
        let write_indent = |buff: &mut String| {
            buff.extend(std::iter::repeat('\t').take(indent));
        };

        match self {
            BlockNode::Paragraph { attributes, children } => {
                write_indent(buff);
                buff.push_str("<p");
                for attr in attributes {
                    write!(buff, " {attr}").unwrap();
                }
                buff.push('>');
                for child in children {
                    child.render(buff);
                }
                buff.push_str("</p>\n");
            }
            BlockNode::Heading {
                level,
                attributes,
                children,
            } => {
                write_indent(buff);
                let tag = level.tag_name();
                buff.push_str(&format!("<{tag}"));
                for attr in attributes {
                    write!(buff, " {attr}").unwrap();
                }
                buff.push('>');
                for child in children {
                    child.render(buff);
                }
                buff.push_str(&format!("</{tag}>\n"));
            }
            BlockNode::Div {
                attributes,
                children,
            } => {
                write_indent(buff);
                buff.push_str("<div");
                for attr in attributes {
                    write!(buff, " {attr}").unwrap();
                }
                buff.push_str(">\n");
                for child in children {
                    child.render(buff, indent + 1);
                }
                write_indent(buff);
                buff.push_str("</div>\n");
            }
            BlockNode::Img { attributes } => {
                write_indent(buff);
                buff.push_str("<img");
                for attr in attributes {
                    write!(buff, " {attr}").unwrap();
                }
                buff.push_str(" />\n");
            }
        }
    }
}

/// ページ直下やdiv直下に配置できる要素
#[derive(Debug, Clone, PartialEq)]
pub enum ContentNode<'s> {
    Block(BlockNode<'s>),
    Inline(InlineNode<'s>),
}

impl<'s> ContentNode<'s> {
    pub fn render(&self, buff: &mut String, indent: usize) {
        match self {
            ContentNode::Block(b) => b.render(buff, indent),
            ContentNode::Inline(i) => {
                buff.extend(std::iter::repeat('\t').take(indent));
                i.render(buff);
                buff.push('\n');
            }
        }
    }
}
