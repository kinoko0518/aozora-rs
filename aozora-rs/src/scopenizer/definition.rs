use crate::prelude::*;
use std::borrow::Cow;

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
            },
        )
    }
}
