use crate::*;
use std::collections::{HashMap, hash_map::Entry};

#[derive(Debug, Default)]
pub struct Scopenized<'s>(pub HashMap<usize, Vec<ScopeKind<'s>>>);

impl<'s> Scopenized<'s> {
    pub fn push_s(&mut self, scope: ScopeKind<'s>) {
        self.0.entry(scope.span.start).or_default().push(scope)
    }

    pub fn push(&mut self, index: Span, deco: Deco<'s>) {
        self.push_s(ScopeKind { deco, span: index });
    }

    pub fn pop(&mut self, index: usize) -> Option<ScopeKind<'s>> {
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
pub struct ScopeKind<'s> {
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
    Text(&'s str),
    Break(Break),
    Odoriji(Odoriji),
    Kunten(&'s str),
    Okurigana(&'s str),
    Figure(Figure<'s>),
}

impl<'s> FlatToken<'s> {
    /// トークンを指定されたインデックスで分割します。
    ///
    /// トークンがインデックスで分割不能な場合、または分割位置がトークンのバイト長より大きい場合は直積の二番目はNoneが返ります。
    pub fn split_at(self, at: usize) -> (FlatToken<'s>, Option<FlatToken<'s>>) {
        if let FlatToken::Text(t) = self {
            if t.bytes().len() < at {
                return (FlatToken::Text(t).into(), None);
            }
            return (
                FlatToken::Text(&t[0..at]),
                Some(FlatToken::Text(&t[at..t.len()])),
            );
        } else {
            return (self.into(), None);
        }
    }
}

impl<'s> Into<Retokenized<'s>> for FlatToken<'s> {
    fn into(self) -> Retokenized<'s> {
        match self {
            FlatToken::Break(b) => Retokenized::Break(b),
            FlatToken::Figure(f) => Retokenized::Figure(f),
            FlatToken::Kunten(k) => Retokenized::Kunten(k),
            FlatToken::Odoriji(o) => Retokenized::Odoriji(o),
            FlatToken::Text(t) => Retokenized::Text(t),
            FlatToken::Okurigana(o) => Retokenized::Okurigana(o),
        }
    }
}
