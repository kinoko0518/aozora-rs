use std::cmp::Ordering;

use crate::{
    AZResult, AZResultC, Deco, RetokenizeError, Retokenized, ScopeAccumulator, Span,
    scopenizer::FlatToken,
};

#[derive(Default, Debug)]
pub enum RetokenizeEvent<'s> {
    FlatTBegin(FlatToken<'s>),
    #[default]
    FlatTEnd,
    DecoBegin(Deco<'s>),
    DecoEnd,
}

type Events<'s> = Vec<(usize, RetokenizeEvent<'s>)>;

pub fn extract_events<'s>(
    scopenized: ScopeAccumulator<'s>,
    flattoken: Vec<(FlatToken<'s>, Span)>,
) -> Events<'s> {
    let mut events = Vec::new();
    for s in scopenized.0.into_iter().map(|(_, i)| i) {
        for s2 in s {
            events.push((s2.span.start, RetokenizeEvent::DecoBegin(s2.deco)));
            events.push((s2.span.end, RetokenizeEvent::DecoEnd));
        }
    }
    for (token, scope) in flattoken {
        events.push((scope.start, RetokenizeEvent::FlatTBegin(token)));
        events.push((scope.end, RetokenizeEvent::FlatTEnd));
    }
    let mut vec = events
        .into_iter()
        .collect::<Vec<(usize, RetokenizeEvent)>>();

    vec.sort_by(|a, b| {
        // まず位置で比較
        let cmp = a.0.cmp(&b.0);
        if cmp != Ordering::Equal {
            return cmp;
        }
        // インデックスが同じ場合はイベントの優先度で比較
        fn priority(e: &RetokenizeEvent) -> u8 {
            match e {
                RetokenizeEvent::FlatTEnd => 0,
                RetokenizeEvent::DecoEnd => 1,
                RetokenizeEvent::DecoBegin(_) => 2,
                RetokenizeEvent::FlatTBegin(_) => 3,
            }
        }
        priority(&a.1).cmp(&priority(&b.1))
    });

    vec
}

#[doc = include_str!("../../docs/retokenize.md")]
pub fn retokenize<'s>(
    flattoken: Vec<(FlatToken<'s>, Span)>,
    scopenized: ScopeAccumulator<'s>,
) -> AZResult<Vec<Retokenized<'s>>, RetokenizeError> {
    // スコープ、平坦トークンの開始、終了を抽出
    let events = extract_events(scopenized, flattoken);
    let mut eacc = AZResultC::default();

    let mut retokenized: Vec<Retokenized> = Vec::new();
    let mut peekable = events.into_iter().peekable();

    // 閉じられていないトークン
    let mut unclosed_token = (Option::None, 0);
    // 閉じられていないタグ
    let mut unclosed_decos: Vec<Deco<'s>> = Vec::new();

    while let Some((i, e)) = peekable.next() {
        match e {
            // 平坦トークン開始
            RetokenizeEvent::FlatTBegin(f) => {
                unclosed_token = (Some(f), i);
            }
            // 平坦トークン終了
            RetokenizeEvent::FlatTEnd => match unclosed_token.0 {
                Some(t) => {
                    // 閉じる
                    retokenized.push(t.into());
                    unclosed_token = (None, i)
                }
                None => eacc.acc_err(
                    // 開始されていないものを閉じたらエラーを蓄積
                    RetokenizeError::InvalidEndOfToken.into(),
                ),
            },
            RetokenizeEvent::DecoBegin(d) => {
                if let (Some(t), unclosed_until) = unclosed_token {
                    // 閉じられていないトークンがあれば一次終了して装飾を開始
                    // トークンを現在位置で分割
                    let (confirmed, unclosed) = t.split_at(i - unclosed_until);
                    // 分割以前を確定、開始タグを挿入、分割以後で未確定トークンを再開
                    retokenized.push(confirmed.into());
                    retokenized.push(Retokenized::DecoBegin(d.clone()));
                    unclosed_decos.push(d);
                    unclosed_token = (unclosed, i);
                } else {
                    // 閉じられていないトークンが無かったときは単純に開始タグを挿入
                    retokenized.push(Retokenized::DecoBegin(d.clone()));
                    unclosed_decos.push(d);
                }
            }
            RetokenizeEvent::DecoEnd => {
                // 装飾を終了
                let popped_deco = unclosed_decos.pop();

                if let (Some(t), unclosed_until) = unclosed_token {
                    // 閉じられていないトークンがあったら
                    if let Some(d) = popped_deco {
                        // 閉じられていないトークンをポップ
                        // 分割して分割以前を確定、終了タグを挿入して分割以後を再開
                        let (confirmed, unclosed) = t.split_at(i - unclosed_until);
                        retokenized.push(confirmed.into());
                        retokenized.push(Retokenized::DecoEnd(d));
                        unclosed_token = (unclosed, i);
                    } else {
                        // 開始されなかったスコープを終了しようとしたらエラー
                        eacc.acc_err(RetokenizeError::InvalidEndOfScope.into());
                        unclosed_token = (Some(t), unclosed_until);
                    }
                } else {
                    // 閉じられていないトークンがない
                    if let Some(d) = popped_deco {
                        // 閉じられていない装飾があれば単に装飾タグを挿入
                        retokenized.push(Retokenized::DecoEnd(d));
                    } else {
                        // 閉じられていない装飾もなければエラー
                        eacc.acc_err(RetokenizeError::InvalidEndOfScope.into());
                    }
                }
            }
        }
    }

    eacc.finally(retokenized)
}
