use super::definition::*;

use crate::scopenizer::conversion::BackRefResult;
use crate::scopenizer::conversion::backref_to_scope;
use crate::scopenizer::error::ScopenizeError;
use crate::tokenizer::*;
use crate::*;

/// ルビや注記などの装飾指定トークンとその周囲のトークンの情報から
/// 注記の適用範囲を確定、どのように記述されていたかの情報を単純化します。
pub fn scopenize<'s>(
    tokens: Vec<Tokenized<'s>>,
) -> AZResult<(ScopeAcc<'s>, ExpAcc<'s>), ScopenizeError> {
    // 改行などのBreakをまたがない注記用のスタック
    let mut inline_stack = Vec::new();

    // Breakをまたぐ注記用のスタック
    let mut stack = Vec::new();

    // 行全体に影響する注記用
    let mut wholeline: Vec<(WholeLine, Span)> = Vec::new();

    // 最終出力用のベクタ
    let mut scopes: ScopeAcc = Vec::new();
    let mut flatten: ExpAcc = Vec::new();

    // エラー蓄積用
    let mut azc = AZResultC::default();

    let mut peekable = tokens.into_iter().peekable();
    while let Some(token) = peekable.next() {
        match token.kind {
            AozoraTokenKind::Text(t) => {
                while let Some(n) = peekable.peek() {
                    match backref_to_scope(&n.kind, (&t, token.span.clone())) {
                        BackRefResult::ScopeConfirmed(s) => {
                            scopes.push(s);
                            peekable.next();
                        }
                        // 前方参照に失敗した場合一個peekableを消費して無視
                        BackRefResult::BackRefFailed => {
                            let err_span = n.span.clone();
                            peekable.next();
                            azc.acc_err(ScopenizeError::BackRefFailed(err_span).into())
                        }
                        BackRefResult::ItWontBackRef => break,
                    }
                }
                flatten.push((Element::Text(t).into(), token.span.clone()));
            }
            AozoraTokenKind::RubyDelimiter => {
                // ルビ区切りが出たら次のトークンがテキスト、次の次のトークンがルビであることを期待する
                if let (Some(t), Some(r)) = (peekable.next(), peekable.next())
                    && let (AozoraTokenKind::Text(text), AozoraTokenKind::Ruby(ruby)) =
                        (t.kind, r.kind)
                {
                    flatten.push((Element::Text(text).into(), t.span.clone()));
                    scopes.push(Scope {
                        deco: Deco::Ruby(ruby),
                        span: t.span,
                    });
                } else {
                    // 修復は難しいので無視
                    azc.acc_err(
                        ScopenizeError::InvalidRubyDelimiterUsage(token.span.clone()).into(),
                    );
                }
            }
            AozoraTokenKind::Annotation(c) => match c {
                Annotation::Sandwiched(s) => match s {
                    Sandwiched::Begin(b) => {
                        inline_stack.push((b, token.span.clone()));
                    }
                    Sandwiched::End(e) => {
                        if inline_stack.is_empty() {
                            // 何もないものを閉じようとするのは単に無視すれば続行可能
                            azc.acc_err(ScopenizeError::IsolatedEndNote(token.span.clone()).into());
                        }
                        while let Some(s) = inline_stack.pop() {
                            let range = s.1.end..token.span.start;
                            if s.0.do_match(&e) {
                                scopes.push(Scope {
                                    deco: s.0.into(),
                                    span: range,
                                });
                            } else {
                                // 交差タグは本来HTMLではエラーであるためエラーを蓄積する
                                // ブラウザ側が自動修復を試みるのでaozora-rs側では特に処理を行わない
                                azc.acc_err(ScopenizeError::CrossingNote(range).into());
                            }
                        }
                    }
                },
                Annotation::Multiline(m) => match m {
                    MultiLine::Begin(b) => {
                        stack.push((b, token.span.clone()));
                    }
                    MultiLine::End(e) => {
                        if stack.is_empty() {
                            // 何もないものを閉じようとするのは単に無視すれば続行可能
                            azc.acc_err(ScopenizeError::IsolatedEndNote(token.span.clone()).into());
                        }
                        while let Some((kind, span)) = stack.pop() {
                            let range = span.end..token.span.start;
                            if kind.do_match(&e) {
                                scopes.push(Scope {
                                    deco: kind.into_deco(),
                                    span: range,
                                });
                            } else {
                                // Sandwichedと同様の対応
                                azc.acc_err(ScopenizeError::CrossingNote(range).into());
                            }
                        }
                    }
                },
                Annotation::Single(s) => {
                    flatten.push((s.into(), token.span.clone()));
                }
                Annotation::BackRef(_) => {
                    // 前方参照型の注記はTextのアームで処理されるため、ここに到達した時点で不正
                    azc.acc_err(ScopenizeError::BackRefFailed(token.span.clone()).into());
                }
                Annotation::WholeLine(w) => {
                    wholeline.push((w, token.span.clone()));
                }
                Annotation::PageDef(p) => {
                    // Retokenize層で処理されるので一旦そのまま置いておく
                    flatten.push((Expression::PageDef(p), token.span.clone()));
                }
                Annotation::Unknown(_) => (), // 不明な注記は一旦無視
            },
            // ルビも前方参照型なのでTextのアームで処理されていることを期待するため
            // このアームに到達した時点で不正
            AozoraTokenKind::Ruby(_) => {
                azc.acc_err(ScopenizeError::BackRefFailed(token.span.clone()).into());
            }
            AozoraTokenKind::Br => {
                flatten.push((Element::Br.into(), token.span.clone()));
                // インライン注記が閉じられていなければエラー
                if !inline_stack.is_empty() {
                    let last = inline_stack.last().unwrap().clone();
                    // inline_stackが空になるまですべて閉じて修復を試みる
                    while let Some(tag) = inline_stack.pop() {
                        let range = tag.1.end..token.span.start;
                        scopes.push(Scope {
                            deco: tag.0.into(),
                            span: range,
                        });
                    }
                    azc.acc_err(
                        ScopenizeError::UnclosedInlineNote(last.1.start..token.span.end).into(),
                    );
                }
                // 行全体注記の範囲を確定
                while let Some(note) = wholeline.pop() {
                    let scope = note.1.end..token.span.start;
                    scopes.push(Scope {
                        deco: note.0.into(),
                        span: scope,
                    });
                }
            }
        }
        if let None = peekable.peek() {
            // 行全体注記の範囲を確定
            while let Some(note) = wholeline.pop() {
                let scope = note.1.end..token.span.end;
                scopes.push(Scope {
                    deco: note.0.into(),
                    span: scope,
                });
            }
        }
    }

    azc.finally((scopes, flatten))
}
