use crate::*;

/// [`Scope`]を束ねたものです。
pub type ScopeAcc<'s> = Vec<Scope<'s>>;
/// [`Expression`]を束ね、[`Span`]情報を付加したものです。
pub type ExpAcc<'s> = Vec<(Expression<'s>, Span)>;

/// 装飾と装飾がかかる範囲を表す構造体です。
#[derive(Debug, PartialEq, Eq)]
pub struct Scope<'s> {
    /// 装飾の種類、およびその固有情報を格納します。
    pub deco: Deco<'s>,
    /// 装飾の影響範囲です。
    pub span: Span,
}

/// 改行、改ページなどの直和です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageBreak {
    /// 「改ページ」に対応
    PageBreak,
    /// 「改丁」に対応
    RectoBreak,
    /// 「改見開き」に対応
    SpreadBreak,
    /// 「改段」に対応
    ColumnBreak,
}

/// [`Scope`]ではない、先頭から順番に並んだ何らかの表現の直和です。
pub enum Expression<'s> {
    /// ページ内の要素に対応します。
    Element(Element<'s>),
    /// 改ページや改丁などに対応します。
    PageBreak(PageBreak),
    /// ページ定義に対応します。
    PageDef(PageDef),
}

/// ルビや注記など、[`Scope`]になるトークンを含まない純粋な単一表現の直和です。
#[derive(Clone, Debug)]
pub enum Element<'s> {
    /// 切り出したテキストに対応します。
    Text(&'s str),
    /// 改行に対応します。
    Br,
    /// 漢文における訓点に対応します。
    Kunten(&'s str),
    /// 漢文における送り仮名に対応します。
    Okurigana(&'s str),
    /// 挿絵や図などに対応します。
    Figure(Figure<'s>),
}

impl<'s> Into<Expression<'s>> for Element<'s> {
    fn into(self) -> Expression<'s> {
        Expression::Element(self)
    }
}

impl<'s> Element<'s> {
    /// トークンを指定されたインデックスで分割します。
    ///
    /// トークンがインデックスで分割不能な場合、または分割位置がトークンのバイト長より大きい場合は直積の二番目はNoneが返ります。
    pub fn split_at(self, at: usize) -> (Element<'s>, Option<Element<'s>>) {
        if let Element::Text(t) = self {
            if t.bytes().len() < at {
                return (Element::Text(t).into(), None);
            }
            return (
                Element::Text(&t[0..at]),
                Some(Element::Text(&t[at..t.len()])),
            );
        } else {
            return (self.into(), None);
        }
    }
}

impl<'s> Into<Retokenized<'s>> for Element<'s> {
    fn into(self) -> Retokenized<'s> {
        match self {
            Element::Br => Retokenized::Br,
            Element::Figure(f) => Retokenized::Figure(f),
            Element::Kunten(k) => Retokenized::Kunten(k),
            Element::Okurigana(o) => Retokenized::Okurigana(o),
            Element::Text(t) => Retokenized::Text(t),
        }
    }
}
