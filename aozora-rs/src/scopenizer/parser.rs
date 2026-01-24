use super::definition::*;
use super::error::*;

use crate::prelude::*;
use crate::scopenizer::conversion::backref_to_scope;
use crate::tokenizer::prelude::*;

pub fn scopenize<'s>(
    tokens: Vec<AozoraToken<'s>>,
    original: &str,
) -> Result<(Vec<FlatToken<'s>>, Vec<Scope<'s>>), miette::Error> {
    // 改行などのBreakをまたがない注記用のスタック
    let mut inline_stack = Vec::new();
    // Breakをまたぐ注記用のスタック
    let mut stack = Vec::new();
    // 最終出力用のベクタ
    let mut scopes = Vec::new();
    let mut flatten = Vec::new();

    let mut peekable = tokens.into_iter().peekable();
    while let Some(token) = peekable.next() {
        match token.kind {
            AozoraTokenKind::Text(t) => {
                while let Some(n) = peekable.peek() {
                    if let Some(s) = backref_to_scope(&n.kind, (&t, token.span.clone())) {
                        scopes.push(s.map_err(|_| BackRefFailed {
                            source_code: original.to_string(),
                            failed_note: token.span.clone(),
                        })?);
                        peekable.next();
                    } else {
                        break;
                    }
                }
                flatten.push(FlatToken::Text(t.clone()));
            }
            AozoraTokenKind::RubyDelimiter => {
                if let (Some(t), Some(r)) = (peekable.next(), peekable.next())
                    && let (AozoraTokenKind::Text(text), AozoraTokenKind::Ruby(ruby)) =
                        (t.kind, r.kind)
                {
                    flatten.push(FlatToken::Text(text));
                    scopes.push(Scope {
                        deco: Deco::Ruby(ruby),
                        span: t.span,
                    });
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
                                scopes.push(Scope {
                                    deco: s.0.into_deco(),
                                    span: range,
                                })
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
                            if kind.do_match(&e) {
                                scopes.push(Scope {
                                    deco: kind.into_deco(),
                                    span,
                                });
                            } else {
                                return Err(CrossingNote {
                                    source_code: original.to_string(),
                                    range: span.start..token.span.end,
                                }
                                .into());
                            }
                        }
                    }
                },
                Note::Single(s) => {
                    flatten.push(s.into_flat_token());
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
                flatten.push(FlatToken::Break(Break::BreakLine));
                if inline_stack.len() != 0 {
                    return Err(UnclosedInlineNote {
                        source_code: original.to_string(),
                        unclosed_area: inline_stack.last().unwrap().1.start..token.span.end,
                    }
                    .into());
                }
            }
            AozoraTokenKind::Odoriji(o) => {
                flatten.push(FlatToken::Odoriji(o));
            }
        }
    }
    Ok((flatten, scopes))
}
