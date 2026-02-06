use std::borrow::Cow;
use std::ops::Range;

use crate::{
    prelude::*,
    retokenizer::prelude::{DecoQueue, Retokenized},
    scopenizer::definition::Scopenized,
};

pub fn retokenize<'s>(
    mut flat: Vec<(FlatToken<'s>, Span)>,
    mut deco: Scopenized<'s>,
) -> Vec<Retokenized<'s>> {
    flat.reverse();
    let mut retokenized: Vec<Retokenized> = Vec::new();
    let mut queue: DecoQueue = DecoQueue::new();

    // Cow文字列をスライスするヘルパー
    let slice_text = |t: &Cow<'s, str>, range: Range<usize>| -> Retokenized<'s> {
        Retokenized::Text(match t {
            Cow::Borrowed(b) => Cow::Borrowed(&b[range]),
            Cow::Owned(o) => Cow::Owned(o[range].to_string()),
        })
    };

    while let Some((token, span)) = flat.pop() {
        match token {
            FlatToken::Text(text) => {
                let mut last_flushed_pos = 0;

                for i in span.clone() {
                    let relative_pos = i - span.start;
                    let mut flushed_at_current_pos = false;

                    // Scopeの終了を処理
                    while let Some(d) = queue.pop(i) {
                        // ここまでのテキストをフラッシュ
                        retokenized.push(slice_text(&text, last_flushed_pos..relative_pos));
                        last_flushed_pos = relative_pos;
                        flushed_at_current_pos = true;

                        retokenized.push(Retokenized::DecoEnd(d));
                    }

                    // Scopeの開始を処理
                    while let Some(d) = deco.pop(i) {
                        // まだフラッシュしていなければ、ここまでのテキストをフラッシュ
                        if !flushed_at_current_pos {
                            retokenized.push(slice_text(&text, last_flushed_pos..relative_pos));
                            last_flushed_pos = relative_pos;
                            flushed_at_current_pos = true;
                        }

                        retokenized.push(Retokenized::DecoBegin(d.deco.clone()));
                        queue.push(d.span.end, d.deco);
                    }
                }

                // 残りのテキストをフラッシュ
                let len = span.end - span.start;
                if last_flushed_pos != len {
                    retokenized.push(slice_text(&text, last_flushed_pos..len));
                }
                while let Some(d) = queue.pop(span.end) {
                    // 終了タグを積む
                    retokenized.push(Retokenized::DecoEnd(d));
                }
            }
            otherwise => {
                for i in span.clone() {
                    // Scopeの終了
                    while let Some(d) = queue.pop(i) {
                        retokenized.push(Retokenized::DecoEnd(d));
                    }
                    // Scopeの開始
                    while let Some(d) = deco.pop(i) {
                        retokenized.push(Retokenized::DecoBegin(d.deco.clone()));
                        queue.push(d.span.end, d.deco);
                    }
                }

                retokenized.push(match otherwise {
                    FlatToken::Text(_) => unreachable!("Handled in the other branch"),
                    FlatToken::Break(b) => Retokenized::Break(b),
                    FlatToken::Figure(f) => Retokenized::Figure(f),
                    FlatToken::Odoriji(o) => Retokenized::Odoriji(o),
                });

                while let Some(d) = queue.pop(span.end) {
                    retokenized.push(Retokenized::DecoEnd(d));
                }
            }
        }
    }
    retokenized
}
