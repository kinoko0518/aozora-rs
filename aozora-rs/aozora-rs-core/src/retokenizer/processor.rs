use std::cmp::Ordering;

use crate::{
    Deco, ExpAcc, Expression, Page, PageBreak, PageDef, RetokenizeError, ScopeAcc,
    scopenizer::Element,
};

#[derive(Default, Debug)]
pub enum RetokenizeEvent<'s> {
    FlatTBegin(Element<'s>),
    #[default]
    FlatTEnd,
    DecoBegin(Deco<'s>),
    DecoEnd,
    PageDef(PageDef),
    PageBreak,
}

type Events<'s> = Vec<(usize, RetokenizeEvent<'s>)>;

fn container_rank(deco: &Deco) -> u8 {
    match deco {
        Deco::Indent(_)
        | Deco::Hanging(_)
        | Deco::VHCentre
        | Deco::HorizontalLayout
        | Deco::Kerning(_) => 0,
        Deco::Grounded | Deco::LowFlying(_) => 1,
        Deco::AHead | Deco::BHead | Deco::CHead => 2,
        _ => 3,
    }
}

struct EventItem<'s> {
    pos: usize,
    event: RetokenizeEvent<'s>,
    span_start: usize,
    span_end: usize,
    rank: u8,
}

pub fn extract_events<'s>(expressions: ExpAcc<'s>, scopenized: ScopeAcc<'s>) -> Events<'s> {
    let mut items = Vec::new();
    for s in scopenized.into_iter() {
        let rank = container_rank(&s.deco);
        items.push(EventItem {
            pos: s.span.start,
            event: RetokenizeEvent::DecoBegin(s.deco),
            span_start: s.span.start,
            span_end: s.span.end,
            rank,
        });
        items.push(EventItem {
            pos: s.span.end,
            event: RetokenizeEvent::DecoEnd,
            span_start: s.span.start,
            span_end: s.span.end,
            rank,
        });
    }
    for (expression, scope) in expressions {
        match expression {
            Expression::Element(e) => {
                items.push(EventItem {
                    pos: scope.start,
                    event: RetokenizeEvent::FlatTBegin(e),
                    span_start: scope.start,
                    span_end: scope.end,
                    rank: 255,
                });
                items.push(EventItem {
                    pos: scope.end,
                    event: RetokenizeEvent::FlatTEnd,
                    span_start: scope.start,
                    span_end: scope.end,
                    rank: 255,
                });
            }
            Expression::PageBreak(b) => {
                items.push(EventItem {
                    pos: scope.start,
                    event: RetokenizeEvent::PageBreak,
                    span_start: scope.start,
                    span_end: scope.end,
                    rank: 255,
                });
                match b {
                    PageBreak::RectoBreak => items.push(EventItem {
                        pos: scope.start,
                        event: RetokenizeEvent::PageDef(PageDef::FromLeft),
                        span_start: scope.start,
                        span_end: scope.end,
                        rank: 255,
                    }),
                    PageBreak::SpreadBreak => items.push(EventItem {
                        pos: scope.start,
                        event: RetokenizeEvent::PageDef(PageDef::FromRight),
                        span_start: scope.start,
                        span_end: scope.end,
                        rank: 255,
                    }),
                    _ => (),
                }
            }
            Expression::PageDef(d) => {
                items.push(EventItem {
                    pos: scope.start,
                    event: RetokenizeEvent::PageDef(d),
                    span_start: scope.start,
                    span_end: scope.end,
                    rank: 255,
                });
            }
        }
    }

    items.sort_by(|a, b| {
        // まず位置で比較
        let cmp = a.pos.cmp(&b.pos);
        if cmp != Ordering::Equal {
            return cmp;
        }
        // インデックスが同じ場合はイベントの優先度で比較
        fn priority(e: &RetokenizeEvent) -> u8 {
            match e {
                RetokenizeEvent::FlatTEnd => 0,
                RetokenizeEvent::DecoEnd => 1,
                RetokenizeEvent::DecoBegin(_) => 2,
                RetokenizeEvent::FlatTBegin(_) => 3,
                RetokenizeEvent::PageDef(_) => 4,
                RetokenizeEvent::PageBreak => 5,
            }
        }
        let p_cmp = priority(&a.event).cmp(&priority(&b.event));
        if p_cmp != Ordering::Equal {
            return p_cmp;
        }

        match (&a.event, &b.event) {
            (RetokenizeEvent::DecoBegin(_), RetokenizeEvent::DecoBegin(_)) => {
                // スコープが外側（span_end が後ろにある）ものを先に開始
                let end_cmp = b.span_end.cmp(&a.span_end);
                if end_cmp != Ordering::Equal {
                    return end_cmp;
                }
                // span_end も同じなら外側コンテナ（rankが小さいもの）を先に開始
                a.rank.cmp(&b.rank)
            }
            (RetokenizeEvent::DecoEnd, RetokenizeEvent::DecoEnd) => {
                // スコープが内側（span_start が後ろにある）ものを先に終了
                let start_cmp = b.span_start.cmp(&a.span_start);
                if start_cmp != Ordering::Equal {
                    return start_cmp;
                }
                // span_start も同じなら内側（rankが大きいもの）を先に終了
                b.rank.cmp(&a.rank)
            }
            _ => Ordering::Equal,
        }
    });

    items.into_iter().map(|item| (item.pos, item.event)).collect()
}

#[doc = include_str!("../../docs/retokenize.md")]
pub fn retokenize<'s>(
    expressions: ExpAcc<'s>,
    scopenized: ScopeAcc<'s>,
) -> (Vec<Page<'s>>, Vec<RetokenizeError>) {
    let mut events = extract_events(expressions, scopenized)
        .into_iter()
        .peekable();
    let mut errors = Vec::new();
    let mut pages = Vec::new();

    while events.peek().is_some() {
        let mut page = Page::default();
        errors.extend(page.retokenize(&mut events).into_iter());
        pages.push(page);
    }

    (pages, errors)
}
