use std::borrow::Cow;

use miette::Diagnostic;
use thiserror::Error;

use crate::{
    Span,
    nihongo::is_kanji,
    tokenizer::{
        AozoraToken, AozoraTokenKind, BackRefKind, BosenKind, BotenKind, Note, Odoriji, Single,
    },
};

#[derive(Debug)]
pub enum Deco<'s> {
    Bold,
    Italic,
    Ruby(&'s str),
    Bosen(BosenKind),
    Boten(BotenKind),
}

#[derive(Debug)]
pub struct Scope<'s> {
    deco: Deco<'s>,
    span: Span,
}

#[derive(Debug, Clone)]
pub enum Break {
    /// 改行に対応
    BreakLine,
    /// 「改ページ」に対応
    PageBreak,
    /// 「改丁」に対応
    RectoBreak,
    /// 「改見開き」に対応
    SpreadBreak,
    /// 「改段」に対応
    ColumnBreak,
}

impl std::fmt::Display for Break {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[#{}]",
            match self {
                Self::BreakLine => "改行",
                Self::PageBreak => "改ページ",
                Self::RectoBreak => "改丁",
                Self::SpreadBreak => "改見開き",
                Self::ColumnBreak => "改段",
            }
        )
    }
}

#[derive(Clone, Debug)]
pub enum FlatToken<'s> {
    Text(Cow<'s, str>),
    Break(Break),
    Odoriji(Odoriji),
}

impl std::fmt::Display for FlatToken<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Text(t) => t.to_string(),
                Self::Break(b) => b.to_string(),
                Self::Odoriji(o) => o.to_string(),
            },
        )
    }
}

impl Single {
    fn into_flat_token<'s>(self) -> FlatToken<'s> {
        match self {
            Self::ColumnBreak => FlatToken::Break(Break::ColumnBreak),
            Self::PageBreak => FlatToken::Break(Break::PageBreak),
            Self::RectoBreak => FlatToken::Break(Break::RectoBreak),
            Self::SpreadBreak => FlatToken::Break(Break::SpreadBreak),
        }
    }
}

#[derive(Error, Debug, Diagnostic)]
#[error("注記が閉じられていません")]
#[diagnostic(
    code(aozora_rs::unclosed_inline_note),
    url(docsrs),
    help("この注記は行末で閉じられる必要があります。行を超えて注記を適用しようとしていませんか？")
)]
struct UnclosedInlineNote {}

#[derive(Error, Debug, Diagnostic)]
#[error("前方参照に失敗しました")]
#[diagnostic(
    code(aozora_rs::backref_failed),
    url(docsrs),
    help(
        "この注記・ルビの前に本文が存在すること、対象文字列が直前に存在していることを確認してください。\n特に、ルビは｜で範囲指定を行わない限り直前に漢字を期待します。"
    )
)]
struct BackRefFailed {
    #[source_code]
    source_code: String,
    #[label("この注記でエラーが発生しています。")]
    failed_note: Span,
}

#[derive(Error, Debug, Diagnostic)]
#[error("ルビデリミタの使い方が不正です")]
#[diagnostic(
    code(aozora_rs::invalid_ruby_delimiter_usage),
    url(docsrs),
    help("ルビデリミタ（｜）とルビの間に本文以外を含めることはできません。")
)]
struct InvalidRubyDelimiterUsage {
    #[source_code]
    source_code: String,
    #[label("この注記でエラーが発生しています。")]
    failed_note: Span,
}

fn backref_to_scope<'s>(
    backref_maybe: &AozoraTokenKind<'s>,
    target: (&str, Span),
) -> Option<Result<Scope<'s>, ()>> {
    match backref_maybe {
        AozoraTokenKind::Ruby(ruby) => Some(Ok(Scope {
            deco: Deco::Ruby(ruby),
            span: {
                // 漢字であり続ける文字数を取得
                let length = target.0.chars().rev().take_while(|c| is_kanji(*c)).count();
                (target.1.end - length)..(target.1.end)
            },
        })),
        AozoraTokenKind::Command(c) => {
            if let Note::BackRef(b) = c {
                Some(Ok(Scope {
                    deco: match b.kind {
                        BackRefKind::Bold => Deco::Bold,
                        BackRefKind::Italic => Deco::Italic,
                        BackRefKind::Bosen(b) => Deco::Bosen(b),
                        BackRefKind::Boten(b) => Deco::Boten(b),
                    },
                    span: if target.0.ends_with(b.range.0) {
                        (target.1.end - b.range.0.len())..target.1.end
                    } else {
                        return Some(Err(()));
                    },
                }))
            } else {
                None
            }
        }
        _ => None,
    }
}

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
