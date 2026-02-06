use crate::prelude::*;
use std::{
    borrow::Cow,
    collections::{HashMap, hash_map::Entry},
};

pub struct Scopenized<'s>(pub HashMap<usize, Vec<Scope<'s>>>);

impl<'s> Scopenized<'s> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn push_s(&mut self, scope: Scope<'s>) {
        self.0
            .entry(scope.span.start)
            .or_insert_with(|| Vec::new())
            .push(scope)
    }

    pub fn push(&mut self, index: Span, deco: Deco<'s>) {
        self.push_s(Scope { deco, span: index });
    }

    pub fn pop(&mut self, index: usize) -> Option<Scope<'s>> {
        match self.0.entry(index) {
            Entry::Occupied(mut entry) => {
                let vec = entry.get_mut();
                let val = vec.pop();
                if vec.is_empty() {
                    entry.remove();
                }
                val
            }
            Entry::Vacant(_) => None,
        }
    }
}

#[derive(Debug)]
pub struct Scope<'s> {
    pub deco: Deco<'s>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Break {
    /// 改行に対応
    BreakLine,
    /// 「改ページ」に対応
    PageBreak,
    /// 「改丁」に対応
    RectoBreak,
    /// 「改見開き」に対応
    SpreadBreak,
    /// 「改段」に対応
    ColumnBreak,
}

#[derive(Clone, Debug)]
pub enum FlatToken<'s> {
    Text(Cow<'s, str>),
    Break(Break),
    Odoriji(Odoriji),
    Figure(Figure<'s>),
}
