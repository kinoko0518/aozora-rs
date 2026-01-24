use crate::{
    nihongo::is_kanji,
    prelude::*,
    scopenizer::definition::{Break, FlatToken, Scope},
    tokenizer::prelude::*,
};

impl Single {
    pub fn into_flat_token<'s>(self) -> FlatToken<'s> {
        match self {
            Self::ColumnBreak => FlatToken::Break(Break::ColumnBreak),
            Self::PageBreak => FlatToken::Break(Break::PageBreak),
            Self::RectoBreak => FlatToken::Break(Break::RectoBreak),
            Self::SpreadBreak => FlatToken::Break(Break::SpreadBreak),
        }
    }
}

pub fn backref_to_scope<'s>(
    backref_maybe: &AozoraTokenKind<'s>,
    target: (&str, Span),
) -> Option<Result<Scope<'s>, ()>> {
    match backref_maybe {
        AozoraTokenKind::Ruby(ruby) => Some(Ok(Scope {
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
        })),
        AozoraTokenKind::Command(c) => {
            if let Note::BackRef(b) = c {
                Some(Ok(Scope {
                    deco: match b.kind {
                        BackRefKind::Bold => Deco::Bold,
                        BackRefKind::Italic => Deco::Italic,
                        BackRefKind::Bosen(b) => Deco::Bosen(b),
                        BackRefKind::Boten(b) => Deco::Boten(b),
                    },
                    span: if target.0.ends_with(b.range.0) {
                        (target.1.end - b.range.0.len())..target.1.end
                    } else {
                        return Some(Err(()));
                    },
                }))
            } else {
                None
            }
        }
        _ => None,
    }
}
