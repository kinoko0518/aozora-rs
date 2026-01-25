use std::collections::HashMap;

use super::definition::*;
use super::error::*;

use crate::prelude::*;
use crate::scopenizer::conversion::backref_to_scope;
use crate::tokenizer::prelude::*;

pub fn scopenize<'s>(
    tokens: Vec<AozoraToken<'s>>,
    original: &str,
) -> Result<(Vec<(FlatToken<'s>, Span)>, HashMap<usize, Scope<'s>>), miette::Error> {
    // 改行などのBreakをまたがない注記用のスタック
    let mut inline_stack = Vec::new();
    // Breakをまたぐ注記用のスタック
    let mut stack = Vec::new();
    // 行全体に影響する注記用
    let mut wholeline: Vec<(WholeLine, Span)> = Vec::new();
    // 最終出力用のベクタ
    let mut scopes = HashMap::new();
    let mut flatten: Vec<(FlatToken, Span)> = Vec::new();

    let mut peekable = tokens.into_iter().peekable();
    while let Some(token) = peekable.next() {
        match token.kind {
            AozoraTokenKind::Text(t) => {
                while let Some(n) = peekable.peek() {
                    if let Some(s) = backref_to_scope(&n.kind, (&t, token.span.clone())) {
                        let scope = token.span.clone();
                        scopes.insert(
                            scope.start,
                            s.map_err(|_| BackRefFailed {
                                source_code: original.to_string(),
                                failed_note: scope,
                            })?,
                        );
                        peekable.next();
                    } else {
                        break;
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
                    scopes.insert(
                        scope.start,
                        Scope {
                            deco: Deco::Ruby(ruby),
                            span: scope,
                        },
                    );
                } else {
                    return Err(InvalidRubyDelimiterUsage {
                        source_code: original.to_string(),
                        failed_note: token.span,
                    }
                    .into());
                }
            }
            AozoraTokenKind::Command(c) => match c {
                Note::Sandwiched(s) => match s {
                    Sandwiched::Begin(b) => {
                        inline_stack.push((b, token.span));
                    }
                    Sandwiched::End(e) => {
                        if inline_stack.is_empty() {
                            return Err(IsolatedEndNote {
                                source_code: original.to_string(),
                                range: token.span,
                            }
                            .into());
                        }
                        while let Some(s) = inline_stack.pop() {
                            let range = s.1.start..token.span.end;
                            if s.0.do_match(&e) {
                                scopes.insert(
                                    range.start,
                                    Scope {
                                        deco: s.0.into_deco(),
                                        span: range,
                                    },
                                );
                            } else {
                                return Err(CrossingNote {
                                    source_code: original.to_string(),
                                    range: range,
                                }
                                .into());
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
                            return Err(IsolatedEndNote {
                                source_code: original.to_string(),
                                range: token.span,
                            }
                            .into());
                        }
                        while let Some((kind, span)) = stack.pop() {
                            let range = span.end..token.span.start;
                            if kind.do_match(&e) {
                                scopes.insert(
                                    range.start,
                                    Scope {
                                        deco: kind.into_deco(),
                                        span: range,
                                    },
                                );
                            } else {
                                return Err(CrossingNote {
                                    source_code: original.to_string(),
                                    range: range,
                                }
                                .into());
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
                    return Err(BackRefFailed {
                        source_code: original.to_string(),
                        failed_note: token.span,
                    }
                    .into());
                }
                Note::WholeLine(w) => {
                    wholeline.push((w, token.span.clone()));
                }
                Note::Unknown(_) => (), // 不明な注記は一旦無視します。
            },
            // ルビも前方参照型なのでTextのアームで処理されていることを期待します。
            // このアームに到達した時点で不正です。
            AozoraTokenKind::Ruby(_) => {
                return Err(BackRefFailed {
                    source_code: original.to_string(),
                    failed_note: token.span,
                }
                .into());
            }
            AozoraTokenKind::Br => {
                flatten.push((FlatToken::Break(Break::BreakLine), token.span.clone()));
                // インライン注記が閉じられていなければエラー
                if inline_stack.len() != 0 {
                    return Err(UnclosedInlineNote {
                        source_code: original.to_string(),
                        unclosed_area: inline_stack.last().unwrap().1.start..token.span.end,
                    }
                    .into());
                }
                // 行全体注記の範囲を確定
                while let Some(note) = wholeline.pop() {
                    let scope = note.1.end..token.span.start;
                    scopes.insert(
                        scope.start,
                        Scope {
                            deco: note.0.into_deco(),
                            span: scope,
                        },
                    );
                }
            }
            AozoraTokenKind::Odoriji(o) => {
                flatten.push((FlatToken::Odoriji(o), token.span));
            }
        }
    }
    Ok((flatten, scopes))
}
