use std::borrow::Cow;

use aozora_rs_core::prelude::*;

use crate::dom::MappedToken;

pub fn into_xhtml<'s>(retokenized: &MappedToken<'s>) -> Cow<'s, str> {
    match &retokenized.content {
        Retokenized::Text(t) => Cow::Owned(format!("<span>{}</span>", t)),
        Retokenized::Odoriji(o) => {
            Cow::Owned(format!("{}〵", if o.has_dakuten { "〴" } else { "〳" }))
        }
        Retokenized::Figure(f) => Cow::Owned(format!(
            "<img src=\"{}\"{}>",
            f.path,
            f.size
                .map(|size| format!(" width=\"{}\" height=\"{}\"", size.0, size.1))
                .unwrap_or("".to_string())
        )),
        Retokenized::Break(b) => match b {
            Break::BreakLine => Cow::Borrowed("<br>"),
            Break::PageBreak | Break::ColumnBreak => Cow::Borrowed(""),
            Break::RectoBreak => Cow::Borrowed(
                "<div style=\"page-break-before: right; break-before: right;\"></div>",
            ),
            Break::SpreadBreak => {
                Cow::Borrowed("<div style=\"page-break-before: left; break-before: left;\"></div>")
            }
        },
        Retokenized::DecoBegin(b) => match &b {
            Deco::AHead => Cow::Owned(format!(
                "<h1 class=\"a_head\" id=\"{}\">",
                retokenized.chapter.as_ref().unwrap().get_id()
            )),
            Deco::BHead => Cow::Owned(format!(
                "<h2 class=\"b_head\" id=\"{}\">",
                retokenized.chapter.as_ref().unwrap().get_id()
            )),
            Deco::CHead => Cow::Owned(format!(
                "<h3 class=\"c_head\" id=\"{}\">",
                retokenized.chapter.as_ref().unwrap().get_id()
            )),
            Deco::Bigger(b) => Cow::Owned(format!(
                "<div style=\"font-size: {}em;\" class=\"{}_bigger\">",
                1.0 + 0.25 * (*b as f32),
                b
            )),
            Deco::Bosen(b) => Cow::Borrowed(match b {
                BosenKind::Plain => "<div style=\"text-decoration-style: solid;\">",
                BosenKind::Double => "<div style=\"text-decoration-style: double;\">",
                BosenKind::Chain => "<div style=\"text-decoration-style: dotted;\">",
                BosenKind::Dashed => "<div style=\"text-decoration-style: dashed;\">",
                BosenKind::Wavy => "<div style=\"text-decoration-style: wavy;\">",
            }),
            Deco::Boten(b) => Cow::Borrowed(match b {
                BotenKind::Sesame => "<div class=\"sesame\">",
                BotenKind::Circle => "<div class=\"circle\">",
                BotenKind::CircleFilled => "<div class=\"circle-filled\">",
                BotenKind::DoubleCircle => "<div class=\"double-circle\">",
                BotenKind::Hebinome => "<div class=\"hebinome\">",
                BotenKind::Crossing => "<div class=\"crossing\">",
                BotenKind::Triangle => "<div class=\"triangle\">",
                BotenKind::TriangleFilled => "<div class=\"triangle-filled\">",
            }),
            Deco::Ruby(_) => Cow::Borrowed("<ruby>"),
            Deco::Bold => Cow::Borrowed("<div class=\"bold\">"),
            Deco::Italic => Cow::Borrowed("<div class=\"italic\">"),
            Deco::Indent(i) => Cow::Owned(format!(
                "<div class=\"indent\" style=\"padding-left: {}em;\">",
                i
            )),
            Deco::Hanging((f, s)) => Cow::Owned(format!(
                "<div class=\"hanging\" style=\"text-indent: {}em;{}\">",
                s,
                if *f == 0 {
                    Cow::Borrowed("")
                } else {
                    Cow::Owned(format!(" padding-left: {}em", f))
                }
            )),
            Deco::Grounded => Cow::Borrowed("<div class=\"grounded\">"),
            Deco::HinV => Cow::Borrowed("<div class=\"hinv\">"),
            Deco::LowFlying(l) => Cow::Owned(format!(
                "<div class=\"lowflying_{} grounded\" style=\"padding-left: {}em;\">",
                l, l,
            )),
            Deco::Mama => Cow::Borrowed("<ruby>"),
            Deco::Smaller(s) => Cow::Owned(format!(
                "<div style=\"font-size: {}em;\" class=\"{}_bigger\">",
                1.0 - 0.25 * (*s as f32),
                s
            )),
            Deco::VHCentre => Cow::Borrowed("<div class=\"vhcentre\">"),
        },
        Retokenized::DecoEnd(e) => match e {
            Deco::Mama => Cow::Borrowed("<rt>ママ</rt></ruby>"),
            Deco::Ruby(r) => Cow::Owned(format!("<rt>{}</rt></ruby>", r)),
            _ => Cow::Borrowed("</div>"),
        },
    }
}
