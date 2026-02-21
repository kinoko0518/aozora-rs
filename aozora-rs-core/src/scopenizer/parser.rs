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
    let mut scopes = Scopenized::new();
    let mut flatten: Vec<(FlatToken, Span)> = Vec::new();
    // エラー蓄積用
    let mut azc = AZResultC::new();

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
                            azc.push(
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
                flatten.push((FlatToken::Text(t.clone()), token.span));
            }
            AozoraTokenKind::RubyDelimiter => {
                // ルビ区切りが出たら次のトークンがテキスト、次の次のトークンが《ルビ》であることを期待します。
                if let (Some(t), Some(r)) = (peekable.next(), peekable.next())
                    && let (AozoraTokenKind::Text(text), AozoraTokenKind::Ruby(ruby)) =
                        (t.kind, r.kind)
                {
                    flatten.push((FlatToken::Text(text), token.span));
                    let scope = t.span;
                    scopes.push(scope, Deco::Ruby(ruby));
                } else {
                    // 修復は難しいので無視
                    azc.push(
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
                            // 何もないものを閉じようとするのは
                            // 単に無視すれば続行可能
                            azc.push(
                                IsolatedEndNote {
                                    source_code: original.to_string(),
                                    range: token.span.clone(),
                                }
                                .into(),
                            );
                        }
                        while let Some(s) = inline_stack.pop() {
                            let range = s.1.start..token.span.end;
                            if s.0.do_match(&e) {
                                scopes.push(range, s.0.into_deco());
                            } else {
                                // 交差タグは本来HTMLではエラーであるためエラーを蓄積する
                                // ブラウザ側が自動修復を試みるのでaozora-rs側では特に処理を行わない
                                azc.push(
                                    CrossingNote {
                                        source_code: original.to_string(),
                                        range: range,
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
                            // 何もないものを閉じようとするのは
                            // 単に無視すれば続行可能
                            azc.push(
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
                                azc.push(
                                    CrossingNote {
                                        source_code: original.to_string(),
                                        range: range,
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
                    // 前方参照型の注記はTextのアームで処理されるため、
                    // ここに到達した時点で不正です。
                    azc.push(
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
                Note::Unknown(_) => (), // 不明な注記は一旦無視します。
            },
            // ルビも前方参照型なのでTextのアームで処理されていることを期待します。
            // このアームに到達した時点で不正です。
            AozoraTokenKind::Ruby(_) => {
                azc.push(
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
                if inline_stack.len() != 0 {
                    // inline_stackが空になるまですべて閉じて修復を試みる
                    while let Some(tag) = inline_stack.pop() {
                        scopes.push(tag.1.end..token.span.start, tag.0.into_deco());
                    }
                    azc.push(
                        UnclosedInlineNote {
                            source_code: original.to_string(),
                            unclosed_area: inline_stack.last().unwrap().1.start..token.span.end,
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
