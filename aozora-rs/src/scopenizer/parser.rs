use super::definition::*;
use super::error::*;

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
                Note::Sandwiched(s) => {
                    inline_stack.push((s, token.span));
                }
                Note::Multiline(m) => {
                    stack.push((m, token.span));
                }
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
            }
            AozoraTokenKind::Odoriji(o) => {
                flatten.push(FlatToken::Odoriji(o));
            }
        }
    }
    Ok((flatten, scopes))
}
