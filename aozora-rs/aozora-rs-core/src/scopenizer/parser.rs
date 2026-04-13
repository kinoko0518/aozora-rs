use super::definition::*;
use super::error::*;

use crate::scopenizer::conversion::BackRefResult;
use crate::scopenizer::conversion::backref_to_scope;
use crate::tokenizer::*;
use crate::*;

pub fn scopenize<'s>(
    tokens: Vec<Tokenized<'s>>,
    original: &str,
) -> AZResult<(Scopenized<'s>, Vec<(FlatToken<'s>, Span)>)> {
    // 改行などのBreakをまたがない注記用のスタック
    let mut inline_stack = Vec::new();
    // Breakをまたぐ注記用のスタック
    let mut stack = Vec::new();
    // 行全体に影響する注記用
    let mut wholeline: Vec<(WholeLine, Span)> = Vec::new();
    // 最終出力用のベクタ
    let mut scopes = Scopenized::default();
    let mut flatten: Vec<(FlatToken, Span)> = Vec::new();
    // エラー蓄積用
    let mut azc = AZResultC::default();

    let mut peekable = tokens.into_iter().peekable();
    while let Some(token) = peekable.next() {
        match token.kind {
            AozoraTokenKind::Text(t) => {
                while let Some(n) = peekable.peek() {
                    let scope = token.span.clone();
                    match backref_to_scope(&n.kind, (&t, token.span.clone())) {
                        BackRefResult::ScopeConfirmed(s) => {
                            scopes.push_s(s);
                            peekable.next();
                        }
                        // 前方参照に失敗した場合一個peekableを消費して無視
                        BackRefResult::BackRefFailed => {
                            peekable.next();
                            azc.acc_err(
                                BackRefFailed {
                                    source_code: original.to_string(),
                                    failed_note: scope,
                                }
                                .into(),
                            )
                        }
                        BackRefResult::ItWontBackRef => break,
                    }
                }
                flatten.push((FlatToken::Text(t), token.span));
            }
            AozoraTokenKind::RubyDelimiter => {
                // ルビ区切りが出たら次のトークンがテキスト、次の次のトークンがルビであることを期待する
                if let (Some(t), Some(r)) = (peekable.next(), peekable.next())
                    && let (AozoraTokenKind::Text(text), AozoraTokenKind::Ruby(ruby)) =
                        (t.kind, r.kind)
                {
                    flatten.push((FlatToken::Text(text), t.span.clone()));
                    scopes.push(t.span, Deco::Ruby(ruby));
                } else {
                    // 修復は難しいので無視
                    azc.acc_err(
                        InvalidRubyDelimiterUsage {
                            source_code: original.to_string(),
                            failed_note: token.span,
                        }
                        .into(),
                    );
                }
            }
            AozoraTokenKind::Note(c) => match c {
                Note::Sandwiched(s) => match s {
                    Sandwiched::Begin(b) => {
                        inline_stack.push((b, token.span));
                    }
                    Sandwiched::End(e) => {
                        if inline_stack.is_empty() {
                            // 何もないものを閉じようとするのは単に無視すれば続行可能
                            azc.acc_err(
                                IsolatedEndNote {
                                    source_code: original.to_string(),
                                    range: token.span.clone(),
                                }
                                .into(),
                            );
                        }
                        while let Some(s) = inline_stack.pop() {
                            let range = s.1.end..token.span.start;
                            if s.0.do_match(&e) {
                                scopes.push(range, s.0.into_deco());
                            } else {
                                // 交差タグは本来HTMLではエラーであるためエラーを蓄積する
                                // ブラウザ側が自動修復を試みるのでaozora-rs側では特に処理を行わない
                                azc.acc_err(
                                    CrossingNote {
                                        source_code: original.to_string(),
                                        range,
                                    }
                                    .into(),
                                );
                            }
                        }
                    }
                },
                Note::Multiline(m) => match m {
                    MultiLine::Begin(b) => {
                        stack.push((b, token.span));
                    }
                    MultiLine::End(e) => {
                        if stack.is_empty() {
                            // 何もないものを閉じようとするのは単に無視すれば続行可能
                            azc.acc_err(
                                IsolatedEndNote {
                                    source_code: original.to_string(),
                                    range: token.span.clone(),
                                }
                                .into(),
                            );
                        }
                        while let Some((kind, span)) = stack.pop() {
                            let range = span.end..token.span.start;
                            if kind.do_match(&e) {
                                scopes.push(range, kind.into_deco());
                            } else {
                                // Sandwichedと同様の対応
                                azc.acc_err(
                                    CrossingNote {
                                        source_code: original.to_string(),
                                        range,
                                    }
                                    .into(),
                                );
                            }
                        }
                    }
                },
                Note::Single(s) => {
                    flatten.push((s.into_flat_token(), token.span));
                }
                Note::BackRef(_) => {
                    // 前方参照型の注記はTextのアームで処理されるため、ここに到達した時点で不正
                    azc.acc_err(
                        BackRefFailed {
                            source_code: original.to_string(),
                            failed_note: token.span,
                        }
                        .into(),
                    );
                }
                Note::WholeLine(w) => {
                    wholeline.push((w, token.span.clone()));
                }
                Note::Unknown(_) => (), // 不明な注記は一旦無視
            },
            // ルビも前方参照型なのでTextのアームで処理されていることを期待するため
            // このアームに到達した時点で不正
            AozoraTokenKind::Ruby(_) => {
                azc.acc_err(
                    BackRefFailed {
                        source_code: original.to_string(),
                        failed_note: token.span,
                    }
                    .into(),
                );
            }
            AozoraTokenKind::Br => {
                flatten.push((FlatToken::Break(Break::BreakLine), token.span.clone()));
                // インライン注記が閉じられていなければエラー
                if !inline_stack.is_empty() {
                    let last = inline_stack.last().unwrap().clone();
                    // inline_stackが空になるまですべて閉じて修復を試みる
                    while let Some(tag) = inline_stack.pop() {
                        scopes.push(tag.1.end..token.span.start, tag.0.into_deco());
                    }
                    azc.acc_err(
                        UnclosedInlineNote {
                            source_code: original.to_string(),
                            unclosed_area: last.1.start..token.span.end,
                        }
                        .into(),
                    );
                }
                // 行全体注記の範囲を確定
                while let Some(note) = wholeline.pop() {
                    let scope = note.1.end..token.span.start;
                    scopes.push(scope, note.0.into_deco());
                }
            }
            AozoraTokenKind::Odoriji(o) => {
                flatten.push((FlatToken::Odoriji(o), token.span));
            }
        }
    }
    azc.finally((scopes, flatten))
}
