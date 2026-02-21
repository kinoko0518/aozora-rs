use crate::{
    nihongo::is_kanji,
    scopenizer::definition::{Break, FlatToken, ScopeKind},
    tokenizer::*,
    *,
};

impl<'s> Single<'s> {
    pub fn into_flat_token(self) -> FlatToken<'s> {
        match self {
            Self::ColumnBreak => FlatToken::Break(Break::ColumnBreak),
            Self::PageBreak => FlatToken::Break(Break::PageBreak),
            Self::RectoBreak => FlatToken::Break(Break::RectoBreak),
            Self::SpreadBreak => FlatToken::Break(Break::SpreadBreak),
            Self::Figure(i) => FlatToken::Figure(i),
        }
    }
}

pub enum BackRefResult<'s> {
    ItWontBackRef,
    BackRefFailed,
    ScopeConfirmed(ScopeKind<'s>),
}

pub fn backref_to_scope<'s>(
    backref_maybe: &AozoraTokenKind<'s>,
    target: (&str, Span),
) -> BackRefResult<'s> {
    match backref_maybe {
        AozoraTokenKind::Ruby(ruby) => BackRefResult::ScopeConfirmed(ScopeKind {
            deco: Deco::Ruby(ruby),
            span: {
                // 漢字であり続けるバイト数を取得
                let length: usize = target
                    .0
                    .chars()
                    .rev()
                    .take_while(|c| is_kanji(*c))
                    .map(|c| c.len_utf8())
                    .sum();
                (target.1.end - length)..(target.1.end)
            },
        }),
        AozoraTokenKind::Note(c) => {
            if let Note::BackRef(b) = c {
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
                    Some((deco, span)) => BackRefResult::ScopeConfirmed(ScopeKind { deco, span }),
                    None => BackRefResult::BackRefFailed,
                }
            } else {
                BackRefResult::ItWontBackRef
            }
        }
        _ => BackRefResult::ItWontBackRef,
    }
}
