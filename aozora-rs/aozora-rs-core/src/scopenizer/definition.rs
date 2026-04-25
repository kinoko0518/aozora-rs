use crate::*;
use std::collections::HashMap;

/// /// 開始地点と[`Scope`]の[`HashMap`]を内部に持ち、[`retokenize`]のために[`Scope`]を蓄積します。
#[derive(Debug, Default)]
pub struct ScopeAccumulator<'s>(pub HashMap<usize, Vec<Scope<'s>>>);

impl<'s> ScopeAccumulator<'s> {
    /// [`Scope`]の`start`を開始地点として挿入します。
    pub fn push_s(&mut self, scope: Scope<'s>) {
        self.0.entry(scope.span.start).or_default().push(scope)
    }

    /// [`Scope`]に加え、開始地点を手動で指定して挿入します。
    pub fn push(&mut self, index: Span, deco: Deco<'s>) {
        self.push_s(Scope { deco, span: index });
    }
}

/// 装飾と装飾がかかる範囲を表す構造体です。
#[derive(Debug)]
pub struct Scope<'s> {
    /// 装飾の種類、およびその固有情報を格納します。
    pub deco: Deco<'s>,
    /// 装飾の影響範囲です。
    pub span: Span,
}

/// 改行、改ページなどの直和です。
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

/// ルビや注記など、[`Scope`]になるトークンを含まない純粋な単一表現の直和です。
#[derive(Clone, Debug)]
pub enum FlatToken<'s> {
    /// 切り出したテキストに対応します。
    Text(&'s str),
    /// 改行、改ページなどに対応します。
    Break(Break),
    /// 漢文における訓点に対応します。
    Kunten(&'s str),
    /// 漢文における送り仮名に対応します。
    Okurigana(&'s str),
    /// 挿絵や図などに対応します。
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
            FlatToken::Text(t) => Retokenized::Text(t),
            FlatToken::Okurigana(o) => Retokenized::Okurigana(o),
        }
    }
}
