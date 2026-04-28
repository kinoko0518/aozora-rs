use std::borrow::Cow;

use aozora_rs_core::{BosenKind, BotenKind, Deco, Retokenized};
use itertools::MultiPeek;
use std::vec::IntoIter;

use crate::xhtmlnize::{
    XHTMLConverter,
    definitions::{XHTMLKind, XHTMLTag},
};

impl<'s> XHTMLConverter<'s> {
    pub(crate) fn handle_deco_begin(
        &mut self,
        peekable: &mut MultiPeek<IntoIter<Retokenized<'s>>>,
        d: Deco<'s>,
    ) {
        match d {
            Deco::AHead => {
                let chapter = self.parse_chapter(peekable, Deco::AHead, |c| c.increament_a());
                self.buff.push(XHTMLTag {
                    kind: XHTMLKind::H1Begin,
                    attributes: vec![
                        Cow::Borrowed("class=\"a_head\""),
                        Cow::Owned(format!("id=\"{}\"", chapter.get_id())),
                    ],
                });
                self.chapters.push(chapter);
            }
            Deco::BHead => {
                let chapter = self.parse_chapter(peekable, Deco::BHead, |c| c.increament_b());
                self.buff.push(XHTMLTag {
                    kind: XHTMLKind::H2Begin,
                    attributes: vec![
                        Cow::Borrowed("class=\"b_head\""),
                        Cow::Owned(format!("id=\"{}\"", chapter.get_id())),
                    ],
                });
                self.chapters.push(chapter);
            }
            Deco::CHead => {
                let chapter = self.parse_chapter(peekable, Deco::CHead, |c| c.increament_c());
                self.buff.push(XHTMLTag {
                    kind: XHTMLKind::H3Begin,
                    attributes: vec![
                        Cow::Borrowed("class=\"c_head\""),
                        Cow::Owned(format!("id=\"{}\"", chapter.get_id())),
                    ],
                });
                self.chapters.push(chapter);
            }
            Deco::Bold => {
                self.buff.push(XHTMLTag {
                    kind: XHTMLKind::SpanBegin,
                    attributes: vec![Cow::Borrowed("class=\"bold\"")],
                });
            }
            Deco::Italic => {
                self.buff.push(XHTMLTag {
                    kind: XHTMLKind::SpanBegin,
                    attributes: vec![Cow::Borrowed("class=\"italic\"")],
                });
            }
            Deco::Ruby(_) | Deco::Mama => {
                self.buff.push(XHTMLTag::from_kind(XHTMLKind::RubyBegin));
            }
            Deco::Bosen(b) => {
                self.buff.push(XHTMLTag {
                    kind: XHTMLKind::SpanBegin,
                    attributes: match b {
                        BosenKind::Chain => vec![Cow::Borrowed("class=\"bosen-chain\"")],
                        BosenKind::Plain => vec![Cow::Borrowed("class=\"bosen-solid\"")],
                        BosenKind::Double => vec![Cow::Borrowed("class=\"bosen-double\"")],
                        BosenKind::Dashed => vec![Cow::Borrowed("class=\"bosen-dashed\"")],
                        BosenKind::Wavy => vec![Cow::Borrowed("class=\"bosen-wavy\"")],
                    },
                });
            }
            Deco::Boten(b) => {
                self.buff.push(XHTMLTag {
                    kind: XHTMLKind::SpanBegin,
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
                });
            }
            Deco::Indent(i) => {
                self.buff.push(XHTMLTag {
                    kind: XHTMLKind::DivBegin,
                    attributes: vec![Cow::Owned(format!(
                        "style=\"padding-inline-start: {}em;\"",
                        i
                    ))],
                });
            }
            Deco::Hanging((h, j)) => {
                self.buff.push(XHTMLTag {
                    kind: XHTMLKind::DivBegin,
                    attributes: vec![Cow::Owned(format!(
                        "style=\"padding-inline-start: {}em; text-indent: {}em;\"",
                        j,
                        (h as i32) - (j as i32)
                    ))],
                });
            }
            Deco::Grounded => {
                self.buff.push(XHTMLTag {
                    kind: XHTMLKind::PBegin,
                    attributes: vec![Cow::Borrowed("class=\"grounded\"")],
                });
            }
            Deco::LowFlying(l) => {
                self.buff.push(XHTMLTag {
                    kind: XHTMLKind::PBegin,
                    attributes: vec![Cow::Owned(format!(
                        "style=\"text-align: right; padding-inline-end: {}em;\"",
                        l
                    ))],
                });
            }
            Deco::HinV => {
                self.buff.push(XHTMLTag {
                    kind: XHTMLKind::SpanBegin,
                    attributes: vec![Cow::Borrowed("class=\"hinv\"")],
                });
            }
            Deco::Bigger(b) => {
                self.buff.push(XHTMLTag {
                    kind: XHTMLKind::SpanBegin,
                    attributes: vec![Cow::Borrowed(match b {
                        1 => "style=\"font-size: large\"",
                        2 => "style=\"font-size: x-large\"",
                        _ => "style=\"font-size: xx-large\"",
                    })],
                });
            }
            Deco::Smaller(b) => {
                self.buff.push(XHTMLTag {
                    kind: XHTMLKind::SpanBegin,
                    attributes: vec![Cow::Borrowed(match b {
                        1 => "style=\"font-size: small\"",
                        2 => "style=\"font-size: x-small\"",
                        _ => "style=\"font-size: xx-small\"",
                    })],
                });
            }
            Deco::VHCentre => {
                self.buff.push(XHTMLTag {
                    kind: XHTMLKind::DivBegin,
                    attributes: vec![Cow::Borrowed("class=\"vhcentre\"")],
                });
            }
            Deco::Warichu => {
                self.buff.push(XHTMLTag {
                    kind: XHTMLKind::SpanBegin,
                    attributes: vec![Cow::Borrowed("class=\"warichu\"")],
                });
            }
            Deco::HorizontalLayout => {
                self.buff.push(XHTMLTag {
                    kind: XHTMLKind::DivBegin,
                    attributes: vec![Cow::Borrowed("class=\"horizontal-block\"")],
                });
            }
            Deco::Kerning(k) => {
                self.buff.push(XHTMLTag {
                    kind: XHTMLKind::DivBegin,
                    attributes: vec![Cow::Owned(format!("style=\"max-inline-size: {}em;\"", k))],
                });
            }
            Deco::Sub => {
                self.buff.push(XHTMLTag::from_kind(XHTMLKind::SubBegin));
            }
            Deco::Sup => {
                self.buff.push(XHTMLTag::from_kind(XHTMLKind::SupBegin));
            }
        }
    }
}
