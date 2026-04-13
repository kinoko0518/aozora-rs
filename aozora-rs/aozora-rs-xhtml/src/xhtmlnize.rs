mod definitions;
mod validate;

use std::{borrow::Cow, vec};

use aozora_rs_core::{BosenKind, BotenKind, Deco, Retokenized, scopenizer::Break};
use itertools::Itertools;

use crate::{
    CDepth, Chapter, XHTMLResult,
    xhtmlnize::{
        definitions::{XHTMLKind, XHTMLTag},
        validate::validate_xhtml,
    },
};

pub fn into_xhtml<'s>(from: Vec<Retokenized<'s>>) -> XHTMLResult {
    // States
    let mut c_depth = CDepth::default();
    let mut peekable = from.into_iter().multipeek();
    let mut buff = Vec::new();

    // Output
    let mut xhtmls = Vec::new();
    let mut dependencies = Vec::new();
    let mut chapters = Vec::new();

    let flush = |buff: &mut Vec<XHTMLTag<'s>>, xhtmls: &mut Vec<Vec<XHTMLTag<'s>>>| {
        xhtmls.push(std::mem::take(buff));
    };
    macro_rules! parse_chapter {
        ($deco_variant:path, $inc_method:ident) => {{
            // CDepthを更新
            c_depth.$inc_method();
            let mut buff = String::new();

            // 章の名前を探索
            while let Some(s) = peekable.peek() {
                match s {
                    Retokenized::DecoEnd($deco_variant) => {
                        break;
                    }
                    Retokenized::Text(t) => {
                        buff.extend(t.chars());
                    }
                    _ => (),
                }
            }
            peekable.reset_peek();
            Chapter {
                xhtml_id: xhtmls.len(),
                name: buff,
                depth: c_depth.clone(),
            }
        }};
    }

    while let Some(token) = peekable.next() {
        match token {
            Retokenized::Text(t) => {
                buff.push(XHTMLTag::from_kind(XHTMLKind::Text(t)));
            }
            Retokenized::Break(b) => match b {
                Break::BreakLine => {
                    buff.push(XHTMLTag::from_kind(XHTMLKind::Br));
                }
                Break::PageBreak | Break::ColumnBreak => flush(&mut buff, &mut xhtmls),
                Break::RectoBreak => {
                    buff.extend([
                        XHTMLTag {
                            kind: XHTMLKind::DivBegin,
                            attributes: vec![Cow::Borrowed(
                                "style=\"page-break-before: right; break-before: right;\"",
                            )],
                        },
                        XHTMLTag::from_kind(XHTMLKind::DivEnd),
                    ]);
                }
                Break::SpreadBreak => {
                    buff.extend([
                        XHTMLTag {
                            kind: XHTMLKind::DivBegin,
                            attributes: vec![Cow::Borrowed(
                                "style=\"page-break-before: left; break-before: left;\"",
                            )],
                        },
                        XHTMLTag::from_kind(XHTMLKind::DivEnd),
                    ]);
                }
            },
            Retokenized::Kunten(k) => {
                buff.extend([
                    XHTMLTag {
                        kind: XHTMLKind::SupBegin,
                        attributes: vec![Cow::Borrowed("class=\"kunten\"")],
                    },
                    XHTMLTag::from_kind(XHTMLKind::Text(k)),
                    XHTMLTag::from_kind(XHTMLKind::SupEnd),
                ]);
            }
            Retokenized::Okurigana(o) => {
                buff.extend([
                    XHTMLTag {
                        kind: XHTMLKind::SupBegin,
                        attributes: vec![Cow::Borrowed("class=\"okurigana\"")],
                    },
                    XHTMLTag::from_kind(XHTMLKind::Text(o)),
                    XHTMLTag::from_kind(XHTMLKind::SupEnd),
                ]);
            }
            Retokenized::Odoriji(o) => {
                let text = if o.has_dakuten { "〴〵" } else { "〳〵" };
                buff.push(XHTMLTag::from_kind(XHTMLKind::Text(text)));
            }
            Retokenized::Figure(f) => {
                let size = f
                    .size
                    .map(|size| format!("width=\"{}\" height=\"{}\"", size.0, size.1))
                    .unwrap_or("".to_string());
                buff.push(XHTMLTag {
                    kind: XHTMLKind::Img,
                    attributes: vec![Cow::Owned(format!("src=\"{}\"", f.path)), Cow::Owned(size)],
                });
                dependencies.push(f.path);
            }
            Retokenized::DecoBegin(d) => match d {
                Deco::AHead => {
                    let chapter = parse_chapter!(Deco::AHead, increament_a);
                    buff.push(XHTMLTag {
                        kind: XHTMLKind::H1Begin,
                        attributes: vec![
                            Cow::Borrowed("class=\"a_head\""),
                            Cow::Owned(format!("id=\"{}\"", chapter.get_id())),
                        ],
                    });
                    chapters.push(chapter);
                }
                Deco::BHead => {
                    let chapter = parse_chapter!(Deco::BHead, increament_b);
                    buff.push(XHTMLTag {
                        kind: XHTMLKind::H2Begin,
                        attributes: vec![
                            Cow::Borrowed("class=\"b_head\""),
                            Cow::Owned(format!("id=\"{}\"", chapter.get_id())),
                        ],
                    });
                    chapters.push(chapter);
                }
                Deco::CHead => {
                    let chapter = parse_chapter!(Deco::CHead, increament_c);
                    buff.push(XHTMLTag {
                        kind: XHTMLKind::H3Begin,
                        attributes: vec![
                            Cow::Borrowed("class=\"c_head\""),
                            Cow::Owned(format!("id=\"{}\"", chapter.get_id())),
                        ],
                    });
                    chapters.push(chapter);
                }
                Deco::Bold => {
                    buff.push(XHTMLTag {
                        kind: XHTMLKind::SpanBegin,
                        attributes: vec![Cow::Borrowed("class=\"bold\"")],
                    });
                }
                Deco::Italic => {
                    buff.push(XHTMLTag {
                        kind: XHTMLKind::SpanBegin,
                        attributes: vec![Cow::Borrowed("class=\"italic\"")],
                    });
                }
                Deco::Ruby(_) | Deco::Mama => {
                    buff.push(XHTMLTag::from_kind(XHTMLKind::RubyBegin));
                }
                Deco::Bosen(b) => {
                    buff.push(XHTMLTag {
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
                    buff.push(XHTMLTag {
                        kind: XHTMLKind::SpanBegin,
                        attributes: match b {
                            BotenKind::Circle => vec![Cow::Borrowed("class=\"circle\"")],
                            BotenKind::CircleFilled => {
                                vec![Cow::Borrowed("class=\"circle-filled\"")]
                            }
                            BotenKind::Sesame => vec![Cow::Borrowed("class=\"sesame\"")],
                            BotenKind::DoubleCircle => {
                                vec![Cow::Borrowed("class=\"double-circle\"")]
                            }
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
                    buff.push(XHTMLTag {
                        kind: XHTMLKind::DivBegin,
                        attributes: vec![Cow::Owned(format!(
                            "style=\"padding-inline-start: {}em;\"",
                            i
                        ))],
                    });
                }
                Deco::Hanging((h, j)) => {
                    buff.push(XHTMLTag {
                        kind: XHTMLKind::DivBegin,
                        attributes: vec![Cow::Owned(format!(
                            "style=\"padding-inline-start: {}em; text-indent: {}em;\"",
                            j,
                            (h as i32) - (j as i32)
                        ))],
                    });
                }
                Deco::Grounded => {
                    buff.push(XHTMLTag {
                        kind: XHTMLKind::PBegin,
                        attributes: vec![Cow::Borrowed("class=\"grounded\"")],
                    });
                }
                Deco::LowFlying(l) => {
                    buff.push(XHTMLTag {
                        kind: XHTMLKind::PBegin,
                        attributes: vec![Cow::Owned(format!(
                            "style=\"text-align: right; padding-inline-end: {}em;\"",
                            l
                        ))],
                    });
                }
                Deco::HinV => {
                    buff.push(XHTMLTag {
                        kind: XHTMLKind::SpanBegin,
                        attributes: vec![Cow::Borrowed("class=\"hinv\"")],
                    });
                }
                Deco::Bigger(b) => {
                    buff.push(XHTMLTag {
                        kind: XHTMLKind::SpanBegin,
                        attributes: vec![Cow::Borrowed(match b {
                            1 => "style=\"font-size: large\"",
                            2 => "style=\"font-size: x-large\"",
                            _ => "style=\"font-size: xx-large\"",
                        })],
                    });
                }
                Deco::Smaller(b) => {
                    buff.push(XHTMLTag {
                        kind: XHTMLKind::SpanBegin,
                        attributes: vec![Cow::Borrowed(match b {
                            1 => "style=\"font-size: small\"",
                            2 => "style=\"font-size: x-small\"",
                            _ => "style=\"font-size: xx-small\"",
                        })],
                    });
                }
                Deco::VHCentre => {
                    buff.push(XHTMLTag {
                        kind: XHTMLKind::DivBegin,
                        attributes: vec![Cow::Borrowed("class=\"vhcentre\"")],
                    });
                }
                Deco::Warichu => {
                    buff.push(XHTMLTag {
                        kind: XHTMLKind::SpanBegin,
                        attributes: vec![Cow::Borrowed("class=\"warichu\"")],
                    });
                }
            },
            Retokenized::DecoEnd(e) => match e {
                Deco::AHead => {
                    buff.push(XHTMLTag::from_kind(XHTMLKind::H1End));
                }
                Deco::BHead => {
                    buff.push(XHTMLTag::from_kind(XHTMLKind::H2End));
                }
                Deco::CHead => {
                    buff.push(XHTMLTag::from_kind(XHTMLKind::H3End));
                }
                Deco::Ruby(r) => {
                    buff.extend(
                        [
                            XHTMLTag::from_kind(XHTMLKind::RtBegin),
                            XHTMLTag::from_kind(XHTMLKind::Text(r)),
                            XHTMLTag::from_kind(XHTMLKind::RtEnd),
                            XHTMLTag::from_kind(XHTMLKind::RubyEnd),
                        ]
                        .into_iter(),
                    );
                }
                Deco::Mama => {
                    buff.extend(
                        [
                            XHTMLTag::from_kind(XHTMLKind::RtBegin),
                            XHTMLTag::from_kind(XHTMLKind::Text("ママ")),
                            XHTMLTag::from_kind(XHTMLKind::RtEnd),
                            XHTMLTag::from_kind(XHTMLKind::RubyEnd),
                        ]
                        .into_iter(),
                    );
                }
                Deco::Indent(_) | Deco::Hanging(_) => {
                    buff.push(XHTMLTag::from_kind(XHTMLKind::DivEnd));
                }
                Deco::Grounded | Deco::LowFlying(_) => {
                    buff.push(XHTMLTag::from_kind(XHTMLKind::PEnd));
                }
                _ => {
                    buff.push(XHTMLTag::from_kind(XHTMLKind::SpanEnd));
                }
            },
        }
    }

    flush(&mut buff, &mut xhtmls);

    XHTMLResult {
        xhtmls: xhtmls
            .into_iter()
            .map(|x| {
                let mut buff = String::new();
                let mut peekable = validate_xhtml(x).into_iter().peekable();
                let mut indent: usize = 0;

                while let Some(s) = peekable.next() {
                    // <br>を独立行として扱うためちょっと補正
                    let fixed_inline =
                        |x: &XHTMLTag| x.kind.is_inline() && !matches!(x.kind, XHTMLKind::Br);
                    let is_inline = fixed_inline(&s);
                    let is_block_end = s.kind.is_block_end();
                    let is_block_begin = s.kind.is_block_begin();

                    if !is_inline && is_block_end {
                        indent = indent.saturating_sub(1);
                    }
                    buff.extend(std::iter::repeat('\t').take(indent));

                    if is_inline {
                        // 同行のインライン要素を消化しきる
                        buff.push_str(&s.into_htmltag());
                        while let Some(inline) = peekable.next_if(fixed_inline) {
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
            })
            .collect(),
        dependency: dependencies.into_iter().map(|d| d.to_string()).collect(),
        chapters,
    }
}
