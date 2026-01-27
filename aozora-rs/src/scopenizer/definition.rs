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

impl std::fmt::Display for Scope<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}[{}..{}]", self.deco, self.span.start, self.span.end)
    }
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

impl std::fmt::Display for Break {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[#{}]",
            match self {
                Self::BreakLine => "改行",
                Self::PageBreak => "改ページ",
                Self::RectoBreak => "改丁",
                Self::SpreadBreak => "改見開き",
                Self::ColumnBreak => "改段",
            }
        )
    }
}

#[derive(Clone, Debug)]
pub enum FlatToken<'s> {
    Text(Cow<'s, str>),
    Break(Break),
    Odoriji(Odoriji),
    Figure(Figure<'s>),
}

impl std::fmt::Display for FlatToken<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Text(t) => t.to_string(),
                Self::Break(b) => b.to_string(),
                Self::Odoriji(o) => o.to_string(),
                Self::Figure(f) => f.to_string(),
            },
        )
    }
}
