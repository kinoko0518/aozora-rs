use std::cmp::Ordering;

use crate::{
    AZResult, AZResultC, Deco, RetokenizeError, Retokenized, Scopenized, Span,
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
    scopenized: Scopenized<'s>,
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
        let cmp = a.0.cmp(&b.0);
        if let Ordering::Equal = cmp {
            match (&a.1, &b.1) {
                // 開始と終了が同じ場所にあった場合、開始が必ず先に来るようにする
                (RetokenizeEvent::DecoBegin(_), RetokenizeEvent::DecoEnd) => Ordering::Greater,
                (RetokenizeEvent::DecoEnd, RetokenizeEvent::DecoBegin(_)) => Ordering::Less,
                (RetokenizeEvent::FlatTBegin(_), RetokenizeEvent::FlatTEnd) => Ordering::Greater,
                (RetokenizeEvent::FlatTEnd, RetokenizeEvent::FlatTBegin(_)) => Ordering::Less,
                _ => Ordering::Equal,
            }
        } else {
            cmp
        }
    });

    vec
}

pub fn retokenize<'s>(
    flattoken: Vec<(FlatToken<'s>, Span)>,
    scopenized: Scopenized<'s>,
) -> AZResult<Vec<Retokenized<'s>>, RetokenizeError> {
    // Retokenizerにおいて、考慮しなければならない状態を列挙する。
    //
    // 1.   トークンの開始地点より前から開始されている注記が存在している
    // 2.   トークンの開始地点で開始される注記が存在している
    // 3.   トークンの途中で開始される注記が存在している
    // 4.   トークンとトークンの隙間で注記が開始されている。
    //
    // 5.   トークンの終了時点で終了されない注記が存在している
    // 6.   トークンの途中で終了する注記が存在している
    // 7.   トークンの開始地点で終了される注記が存在している
    // 8.   トークンとトークンの隙間で注記が終了されている。
    //
    // Retokenizerは注記の影響範囲を開始タグ・終了タグに変換し、
    // トークン列と注記の影響範囲の直積を再トークン列にフラット化する。
    //
    // 目的の達成のため、以下のアルゴリズムで目的を達成する。
    //
    // 1.   トークンの開始地点と終了地点、注記の開始地点と終了地点を抽出する。
    // 2.   トークンの開始・終了、注記の開始・終了は同時には起こらないため、
    //      トークンと注記それぞれについてBegin・End・Noneから成る直和に変換を行い、
    //      (at, token_state, scope_state)のベクタに変換する。
    // 3.   スコープ開始・終了を受け取ったとき、閉じられていないトークンを途中で切り、
    //      開始・終了タグを積み、閉じられていないトークンを再開する。
    // 4.   トークン開始を受け取ったとき、スタックに閉じられていないトークンとして積む。
    // 5.   トークン終了を受け取ったとき、スタックの一番上のトークンを確定して積む。

    let events = extract_events(scopenized, flattoken);

    let mut eacc = AZResultC::default();

    let mut retokenized: Vec<Retokenized> = Vec::new();
    let mut peekable = events.into_iter().peekable();

    let mut unclosed_token = (Option::None, 0);
    let mut unclosed_decos: Vec<Deco<'s>> = Vec::new();

    while let Some((i, e)) = peekable.next() {
        match e {
            RetokenizeEvent::FlatTBegin(f) => {
                unclosed_token = (Some(f), i);
            }
            RetokenizeEvent::FlatTEnd => match unclosed_token.0 {
                Some(t) => {
                    retokenized.push(t.into());
                    unclosed_token = (None, i)
                }
                None => eacc.acc_err(RetokenizeError::InvalidEndOfToken.into()),
            },
            RetokenizeEvent::DecoBegin(d) => {
                if let (Some(t), unclosed_until) = unclosed_token {
                    let (confirmed, unclosed) = t.split_at(i - unclosed_until);
                    retokenized.push(confirmed.into());
                    retokenized.push(Retokenized::DecoBegin(d.clone()));
                    unclosed_decos.push(d);
                    unclosed_token = (unclosed, i);
                } else {
                    retokenized.push(Retokenized::DecoBegin(d.clone()));
                    unclosed_decos.push(d);
                }
            }
            RetokenizeEvent::DecoEnd => {
                let popped_deco = unclosed_decos.pop();

                if let (Some(t), unclosed_until) = unclosed_token {
                    if let Some(d) = popped_deco {
                        let (confirmed, unclosed) = t.split_at(i - unclosed_until);
                        retokenized.push(confirmed.into());
                        retokenized.push(Retokenized::DecoEnd(d));
                        unclosed_token = (unclosed, i);
                    } else {
                        eacc.acc_err(RetokenizeError::InvalidEndOfScope.into());
                        unclosed_token = (Some(t), unclosed_until);
                    }
                } else {
                    if let Some(d) = popped_deco {
                        retokenized.push(Retokenized::DecoEnd(d));
                    } else {
                        eacc.acc_err(RetokenizeError::InvalidEndOfScope.into());
                    }
                }
            }
        }
    }

    eacc.finally(retokenized)
}
