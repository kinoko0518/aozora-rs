use std::borrow::Cow;
use std::vec::IntoIter;

use aozora_rs_core::{BosenKind, BotenKind, Deco, Retokenized};
use itertools::MultiPeek;

use crate::{
    CDepth, Chapter,
    xhtmlnize::{
        ast::HeadingLevel,
        builder::{TreeBuilder, frame::ContainerFrame},
    },
};

impl<'s> TreeBuilder<'s> {
    pub(crate) fn parse_chapter<F>(
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

    pub(crate) fn handle_deco_begin(
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

    pub(crate) fn handle_deco_end(&mut self, e: Deco<'s>) {
        match e {
            Deco::Ruby(r) => self.close_ruby(Cow::Borrowed(r)),
            Deco::Mama => self.close_ruby(Cow::Borrowed("ママ")),
            _ => self.close_frame_and_attach(),
        }
    }
}
