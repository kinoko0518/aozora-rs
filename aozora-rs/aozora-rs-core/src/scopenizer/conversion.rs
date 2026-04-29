use crate::{nihongo::is_kanji, scopenizer::definition::Scope, tokenizer::*, *};

impl<'s> From<Single<'s>> for Expression<'s> {
    fn from(val: Single<'s>) -> Self {
        match val {
            Single::ColumnBreak => Expression::PageBreak(PageBreak::ColumnBreak),
            Single::PageBreak => Expression::PageBreak(PageBreak::PageBreak),
            Single::RectoBreak => Expression::PageBreak(PageBreak::RectoBreak),
            Single::SpreadBreak => Expression::PageBreak(PageBreak::SpreadBreak),
            Single::Kundoku(k) => Expression::Element(Element::Kunten(k)),
            Single::Okurigana(o) => Expression::Element(Element::Okurigana(o)),
            Single::Figure(i) => Expression::Element(Element::Figure(i)),
        }
    }
}

pub enum BackRefResult<'s> {
    ItWontBackRef,
    BackRefFailed,
    ScopeConfirmed(Scope<'s>),
}

pub fn backref_to_scope<'s>(
    backref_maybe: &AozoraTokenKind<'s>,
    target: (&str, Span),
) -> BackRefResult<'s> {
    match backref_maybe {
        AozoraTokenKind::Ruby(ruby) => {
            // 漢字、またはアルファベットであり続けるバイト数を取得
            let length: usize = target
                .0
                .chars()
                .rev()
                .take_while(|c| is_kanji(*c) || c.is_ascii_alphabetic())
                .map(|c| c.len_utf8())
                .sum();
            if length > 0 {
                BackRefResult::ScopeConfirmed(Scope {
                    deco: Deco::Ruby(ruby),
                    span: (target.1.end - length)..(target.1.end),
                })
            } else {
                BackRefResult::BackRefFailed
            }
        }
        AozoraTokenKind::Annotation(c) => {
            if let Annotation::BackRef(b) = c {
                match match b.kind {
                    BackRefKind::Bold => Some(Deco::Bold),
                    BackRefKind::Italic => Some(Deco::Italic),
                    BackRefKind::Bosen(b) => Some(Deco::Bosen(b)),
                    BackRefKind::Boten(b) => Some(Deco::Boten(b)),
                    BackRefKind::AHead => Some(Deco::AHead),
                    BackRefKind::BHead => Some(Deco::BHead),
                    BackRefKind::CHead => Some(Deco::CHead),
                    BackRefKind::HinV => Some(Deco::HinV),
                    BackRefKind::Mama => Some(Deco::Mama),
                    BackRefKind::Big(size) => Some(Deco::Bigger(size)),
                    BackRefKind::Small(size) => Some(Deco::Smaller(size)),
                    BackRefKind::Sub => Some(Deco::Sub),
                    BackRefKind::Sup => Some(Deco::Sup),
                    BackRefKind::Note(n) => Some(Deco::Ruby(n)),
                    BackRefKind::Variation(_) => None,
                }
                .and_then(|deco| {
                    let span = if target.0.ends_with(b.range.0) {
                        Some((target.1.end - b.range.0.len())..target.1.end)
                    } else {
                        None
                    };
                    span.map(|s| (deco, s))
                }) {
                    Some((deco, span)) => BackRefResult::ScopeConfirmed(Scope { deco, span }),
                    None => BackRefResult::BackRefFailed,
                }
            } else {
                BackRefResult::ItWontBackRef
            }
        }
        _ => BackRefResult::ItWontBackRef,
    }
}
