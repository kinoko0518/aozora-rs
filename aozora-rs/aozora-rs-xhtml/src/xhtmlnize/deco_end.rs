use aozora_rs_core::Deco;

use crate::xhtmlnize::{
    XHTMLConverter,
    definitions::{XHTMLKind, XHTMLTag},
};

impl<'s> XHTMLConverter<'s> {
    pub(crate) fn handle_deco_end(&mut self, e: Deco<'s>) {
        match e {
            Deco::AHead => self.buff.push(XHTMLTag::from_kind(XHTMLKind::H1End)),
            Deco::BHead => self.buff.push(XHTMLTag::from_kind(XHTMLKind::H2End)),
            Deco::CHead => self.buff.push(XHTMLTag::from_kind(XHTMLKind::H3End)),
            Deco::Ruby(r) => {
                self.buff.extend([
                    XHTMLTag::from_kind(XHTMLKind::RtBegin),
                    XHTMLTag::from_kind(XHTMLKind::Text(r)),
                    XHTMLTag::from_kind(XHTMLKind::RtEnd),
                    XHTMLTag::from_kind(XHTMLKind::RubyEnd),
                ]);
            }
            Deco::Mama => {
                self.buff.extend([
                    XHTMLTag::from_kind(XHTMLKind::RtBegin),
                    XHTMLTag::from_kind(XHTMLKind::Text("ママ")),
                    XHTMLTag::from_kind(XHTMLKind::RtEnd),
                    XHTMLTag::from_kind(XHTMLKind::RubyEnd),
                ]);
            }
            Deco::Indent(_) | Deco::Hanging(_) | Deco::HorizontalLayout | Deco::Kerning(_) => {
                self.buff.push(XHTMLTag::from_kind(XHTMLKind::DivEnd));
            }
            Deco::Grounded | Deco::LowFlying(_) => {
                self.buff.push(XHTMLTag::from_kind(XHTMLKind::PEnd));
            }
            Deco::Sup => self.buff.push(XHTMLTag::from_kind(XHTMLKind::SupEnd)),
            Deco::Sub => self.buff.push(XHTMLTag::from_kind(XHTMLKind::SubEnd)),
            _ => self.buff.push(XHTMLTag::from_kind(XHTMLKind::SpanEnd)),
        }
    }
}
