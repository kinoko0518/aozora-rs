use std::borrow::Cow;

use aozora_rs_core::prelude::*;

#[allow(unused)]
pub enum XHTMLRequest<'s> {
    None,
    Eof,
    Dependency(&'s str),
    Isolated,
}

pub fn into_xhtml<'s>(retokenized: Retokenized<'s>) -> (Cow<'s, str>, XHTMLRequest<'s>) {
    match retokenized {
        Retokenized::Text(t) => (
            Cow::Owned(format!("<span>{}</span>", t)),
            XHTMLRequest::None,
        ),
        Retokenized::Odoriji(o) => (
            Cow::Owned(format!("{}〵", if o.has_dakuten { "〴" } else { "〳" })),
            XHTMLRequest::None,
        ),
        Retokenized::Figure(f) => (
            Cow::Owned(format!(
                "<img src=\"{}\"{}>",
                f.path,
                f.size
                    .map(|size| format!(" width=\"{}\" height=\"{}\"", size.0, size.1))
                    .unwrap_or("".to_string())
            )),
            XHTMLRequest::Dependency(f.path),
        ),
        Retokenized::Break(b) => match b {
            Break::BreakLine => (Cow::Borrowed("<br>"), XHTMLRequest::None),
            Break::PageBreak | Break::ColumnBreak => (Cow::Borrowed(""), XHTMLRequest::Eof),
            Break::RectoBreak => (
                Cow::Borrowed(
                    "<div style=\"page-break-before: right; break-before: right;\"></div>",
                ),
                XHTMLRequest::None,
            ),
            Break::SpreadBreak => (
                Cow::Borrowed("<div style=\"page-break-before: left; break-before: left;\"></div>"),
                XHTMLRequest::None,
            ),
        },
        Retokenized::DecoBegin(b) => match b {
            Deco::AHead => (Cow::Borrowed("<h1 class=\"a_head\">"), XHTMLRequest::None),
            Deco::BHead => (Cow::Borrowed("<h2 class=\"b_head\">"), XHTMLRequest::None),
            Deco::CHead => (Cow::Borrowed("<h3 class=\"c_head\">"), XHTMLRequest::None),
            Deco::Bigger(b) => (
                Cow::Owned(format!(
                    "<div style=\"font-size: {}em;\" class=\"{}_bigger\">",
                    1.0 + 0.25 * (b as f32),
                    b
                )),
                XHTMLRequest::None,
            ),
            Deco::Bosen(b) => (
                Cow::Borrowed(match b {
                    BosenKind::Plain => "<div style=\"text-decoration-style: solid;\">",
                    BosenKind::Double => "<div style=\"text-decoration-style: double;\">",
                    BosenKind::Chain => "<div style=\"text-decoration-style: dotted;\">",
                    BosenKind::Dashed => "<div style=\"text-decoration-style: dashed;\">",
                    BosenKind::Wavy => "<div style=\"text-decoration-style: wavy;\">",
                }),
                XHTMLRequest::None,
            ),
            Deco::Boten(b) => (
                Cow::Borrowed(match b {
                    BotenKind::Sesame => "<div class=\"sesame\">",
                    BotenKind::Circle => "<div class=\"circle\">",
                    BotenKind::CircleFilled => "<div class=\"circle-filled\">",
                    BotenKind::DoubleCircle => "<div class=\"double-circle\">",
                    BotenKind::Hebinome => "<div class=\"hebinome\">",
                    BotenKind::Crossing => "<div class=\"crossing\">",
                    BotenKind::Triangle => "<div class=\"triangle\">",
                    BotenKind::TriangleFilled => "<div class=\"triangle-filled\">",
                }),
                XHTMLRequest::None,
            ),
            Deco::Ruby(_) => (Cow::Borrowed("<ruby>"), XHTMLRequest::None),
            Deco::Bold => (Cow::Borrowed("<div class=\"bold\">"), XHTMLRequest::None),
            Deco::Italic => (Cow::Borrowed("<div class=\"italic\">"), XHTMLRequest::None),
            Deco::Indent(i) => (
                Cow::Owned(format!(
                    "<div class=\"indent\" style=\"padding-left: {}em;\">",
                    i
                )),
                XHTMLRequest::None,
            ),
            Deco::Hanging((f, s)) => (
                Cow::Owned(format!(
                    "<div class=\"hanging\" style=\"text-indent: {}em;{}\">",
                    s,
                    if f == 0 {
                        Cow::Borrowed("")
                    } else {
                        Cow::Owned(format!(" padding-left: {}em", f))
                    }
                )),
                XHTMLRequest::None,
            ),
            Deco::Grounded => (
                Cow::Borrowed("<div class=\"grounded\">"),
                XHTMLRequest::None,
            ),
            Deco::HinV => (Cow::Borrowed("<div class=\"hinv\">"), XHTMLRequest::None),
            Deco::LowFlying(l) => (
                Cow::Owned(format!(
                    "<div class=\"lowflying_{} grounded\" style=\"padding-left: {}em;\">",
                    l, l,
                )),
                XHTMLRequest::None,
            ),
            Deco::Mama => (Cow::Borrowed("<ruby>"), XHTMLRequest::None),
            Deco::Smaller(s) => (
                Cow::Owned(format!(
                    "<div style=\"font-size: {}em;\" class=\"{}_bigger\">",
                    1.0 - 0.25 * (s as f32),
                    s
                )),
                XHTMLRequest::None,
            ),
            Deco::VHCentre => (
                Cow::Borrowed("<div class=\"vhcentre\">"),
                XHTMLRequest::Isolated,
            ),
        },
        Retokenized::DecoEnd(e) => match e {
            Deco::Mama => (Cow::Borrowed("<rt>ママ</rt></ruby>"), XHTMLRequest::None),
            Deco::Ruby(r) => (
                Cow::Owned(format!("<rt>{}</rt></ruby>", r)),
                XHTMLRequest::None,
            ),
            _ => (Cow::Borrowed("</div>"), XHTMLRequest::None),
        },
    }
}
