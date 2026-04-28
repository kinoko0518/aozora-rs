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

pub fn extract_events<'s>(expressions: ExpAcc<'s>, scopenized: ScopeAcc<'s>) -> Events<'s> {
    let mut events = Vec::new();
    for s in scopenized.into_iter() {
        events.push((s.span.start, RetokenizeEvent::DecoBegin(s.deco)));
        events.push((s.span.end, RetokenizeEvent::DecoEnd));
    }
    for (expression, scope) in expressions {
        match expression {
            Expression::Element(e) => {
                events.push((scope.start, RetokenizeEvent::FlatTBegin(e)));
                events.push((scope.end, RetokenizeEvent::FlatTEnd));
            }
            Expression::PageBreak(b) => {
                events.push((scope.start, RetokenizeEvent::PageBreak));
                match b {
                    PageBreak::RectoBreak => {
                        events.push((scope.start, RetokenizeEvent::PageDef(PageDef::FromLeft)))
                    }
                    PageBreak::SpreadBreak => {
                        events.push((scope.start, RetokenizeEvent::PageDef(PageDef::FromRight)))
                    }
                    _ => (),
                }
            }
            Expression::PageDef(d) => {
                events.push((scope.start, RetokenizeEvent::PageDef(d)));
            }
        }
    }
    let mut vec = events
        .into_iter()
        .collect::<Vec<(usize, RetokenizeEvent)>>();

    vec.sort_by(|a, b| {
        // まず位置で比較
        let cmp = a.0.cmp(&b.0);
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
        priority(&a.1).cmp(&priority(&b.1))
    });

    vec
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
