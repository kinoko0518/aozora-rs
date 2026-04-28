mod pages;

use crate::*;

pub use pages::Page;

/// 開始タグ・要素・終了タグで構成される、HTMLライクな中間表現です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Retokenized<'s> {
    /// 切り出したテキストに対応します。
    Text(&'s str),
    /// 漢文における訓点に対応します。
    Kunten(&'s str),
    /// 漢文における送り仮名に対応します。
    Okurigana(&'s str),
    /// 改行に対応します。
    Br,
    /// 挿絵、図などに対応します。
    Figure(Figure<'s>),
    /// 装飾の開始に対応します。
    DecoBegin(Deco<'s>),
    /// 装飾の終了に対応します。
    DecoEnd(Deco<'s>),
}

/// 再トークン化時に発生しうるエラーの直和です。
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum RetokenizeError {
    /// トークンを閉じろという命令が出た際、閉じるトークンがなければこのエラーが生じます。
    #[default]
    InvalidEndOfToken,
    /// スコープを閉じろという命令が出た際、閉じるスコープがなければこのエラーが生じます。
    InvalidEndOfScope,
}

impl std::fmt::Display for RetokenizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::InvalidEndOfScope => "スコープの終了地点が不正です。これは内部的なエラーです",
                Self::InvalidEndOfToken => "トークンの終了地点が不正です。これは内部的なエラーです",
            }
        )
    }
}

impl Retokenized<'_> {
    /// 要素が可視要素かを真理値で返却します。
    pub fn is_visible(&self) -> bool {
        match self {
            Self::Kunten(k) => !k.is_empty(),
            Self::Okurigana(o) => !o.is_empty(),
            Self::DecoBegin(_) => false,
            Self::DecoEnd(_) => false,
            _ => true,
        }
    }
}

/// ページの要素が必ず左から開始する、右から開始するなどを指定します。
pub enum PageBegin {
    /// 要素は左ページから開始します。
    Left,
    /// 要素は右ページから開始します。
    Right,
    /// 特に指定を行いません。
    Whatever,
}
