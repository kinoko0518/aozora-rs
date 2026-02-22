use std::borrow::Cow;

use aozora_rs_core::{scopenizer::Break, *};

use crate::dom::MappedToken;

pub fn into_xhtml<'s>(mapped: &MappedToken<'s>) -> Cow<'s, str> {
    match &mapped.content {
        Retokenized::Text(t) => Cow::Owned(format!("<span>{}</span>", t.replace('\r', ""))),
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
            Break::BreakLine => Cow::Borrowed("<br />"),
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
                mapped.chapter.as_ref().unwrap().get_id()
            )),
            Deco::BHead => Cow::Owned(format!(
                "<h2 class=\"b_head\" id=\"{}\">",
                mapped.chapter.as_ref().unwrap().get_id()
            )),
            Deco::CHead => Cow::Owned(format!(
                "<h3 class=\"c_head\" id=\"{}\">",
                mapped.chapter.as_ref().unwrap().get_id()
            )),
            Deco::Bigger(b) => Cow::Owned(format!(
                "<span style=\"font-size: {}em;\" class=\"{}_bigger\">",
                1.0 + 0.25 * (*b as f32),
                b
            )),
            Deco::Bosen(b) => Cow::Borrowed(match b {
                BosenKind::Plain => "<span style=\"text-decoration-style: solid;\">",
                BosenKind::Double => "<span style=\"text-decoration-style: double;\">",
                BosenKind::Chain => "<span style=\"text-decoration-style: dotted;\">",
                BosenKind::Dashed => "<span style=\"text-decoration-style: dashed;\">",
                BosenKind::Wavy => "<span style=\"text-decoration-style: wavy;\">",
            }),
            Deco::Boten(b) => Cow::Borrowed(match b {
                BotenKind::Sesame => "<span class=\"sesame\">",
                BotenKind::Circle => "<span class=\"circle\">",
                BotenKind::CircleFilled => "<span class=\"circle-filled\">",
                BotenKind::DoubleCircle => "<span class=\"double-circle\">",
                BotenKind::Hebinome => "<span class=\"hebinome\">",
                BotenKind::Crossing => "<span class=\"crossing\">",
                BotenKind::Triangle => "<span class=\"triangle\">",
                BotenKind::TriangleFilled => "<span class=\"triangle-filled\">",
            }),
            Deco::Ruby(_) => Cow::Borrowed("<ruby>"),
            Deco::Bold => Cow::Borrowed("<span class=\"bold\">"),
            Deco::Italic => Cow::Borrowed("<span class=\"italic\">"),
            Deco::Mama => Cow::Borrowed("<ruby>"),
            Deco::Smaller(s) => Cow::Owned(format!(
                "<span style=\"font-size: {}em;\" class=\"{}_smaller\">",
                1.0 - 0.25 * (*s as f32),
                s
            )),
            Deco::Indent(i) => Cow::Owned(format!(
                "<div class=\"indent\" style=\"padding-inline: {}em;\">",
                i
            )),
            Deco::Hanging((f, s)) => Cow::Owned(format!(
                "<div class=\"hanging\" style=\"text-indent: {}em;{}\">",
                s,
                if *f == 0 {
                    Cow::Borrowed("")
                } else {
                    Cow::Owned(format!(" padding-inline: {}em", f))
                }
            )),
            Deco::Grounded => Cow::Borrowed("<div class=\"grounded\">"),
            Deco::HinV => Cow::Borrowed("<div class=\"hinv\">"),
            Deco::LowFlying(l) => Cow::Owned(format!(
                "<div class=\"lowflying_{} grounded\" style=\"padding-inline: {}em;\">",
                l, l,
            )),
            Deco::VHCentre => Cow::Borrowed("<div class=\"vhcentre\">"),
        },
        Retokenized::DecoEnd(e) => match e {
            Deco::Mama => Cow::Borrowed("<rt>ママ</rt></ruby>"),
            Deco::Ruby(r) => Cow::Owned(format!("<rt>{}</rt></ruby>", r)),
            Deco::AHead => Cow::Borrowed("</h1>"),
            Deco::BHead => Cow::Borrowed("</h2>"),
            Deco::CHead => Cow::Borrowed("</h3>"),
            Deco::Indent(_)
            | Deco::Hanging(_)
            | Deco::VHCentre
            | Deco::LowFlying(_)
            | Deco::HinV
            | Deco::Grounded => Cow::Borrowed("</div>"),
            _ => Cow::Borrowed("</span>"),
        },
    }
}
