use super::Span;
use crate::{nihongo::is_kanji, tokenizer::command::Note};

/// 最終的に本文になるテキストを保管する構造体です。
pub struct Text<'s> {
    text: &'s str,
    slice: Span,
}

impl<'s> Text<'s> {
    /// textの最後から走査、漢字である範囲を返すヘルパ関数です。
    /// ｜は別トークンとして切り出しているため、考慮しません。
    pub fn ruby_target_from_end(&self) -> Option<Span> {
        let len: usize = self
            .text
            .chars()
            .rev()
            .take_while(|c| is_kanji(*c))
            .map(|c| c.len_utf8())
            .sum();
        if len == 0 {
            None
        } else {
            Some((self.slice.end - len)..self.slice.end)
        }
    }
}

pub struct Ruby<'s> {
    target: &'s str,
    span: Span,
}

pub struct Odoriji {
    has_dakuten: bool,
    span: Span,
}

pub enum AozoraToken<'s> {
    Text(Text<'s>),
    Ruby(Ruby<'s>),
    RubyDelimiter(Span),
    Note(Note<'s>),
    Odoriji(Odoriji),
}
