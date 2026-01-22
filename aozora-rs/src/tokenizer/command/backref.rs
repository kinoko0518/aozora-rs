//! aozorahackの青空文庫書式のドキュメントに記載されているところの、前方参照型のパースを行うモジュールです。
//!　ドキュメントは[こちら](https://github.com/aozorahack/specs/blob/master/aozora-text.md#%E5%89%8D%E6%96%B9%E5%8F%82%E7%85%A7%E5%9E%8B%E3%81%A8%E9%96%8B%E5%A7%8B%E7%B5%82%E4%BA%86%E5%9E%8B)
//! から確認できます。

use crate::tokenizer::{
    Span,
    command::definitions::{BosenKind, BotenKind},
};

struct BackRefSpec<'s> {
    target: &'s str,
    spec: Span,
}

pub enum BackRefNote<'s> {
    Bold(BackRefSpec<'s>),
    Italic(BackRefSpec<'s>),
    Boten((BackRefSpec<'s>, BotenKind)),
    Bosen((BackRefSpec<'s>, BosenKind)),
}
