use crate::Span;
use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
#[error("注記が閉じられていません")]
#[diagnostic(
    code(aozora_rs::unclosed_inline_note),
    url(docsrs),
    help("この注記は行末で閉じられる必要があります。行を超えて注記を適用しようとしていませんか？")
)]
pub struct UnclosedInlineNote {}

#[derive(Error, Debug, Diagnostic)]
#[error("前方参照に失敗しました")]
#[diagnostic(
    code(aozora_rs::backref_failed),
    url(docsrs),
    help(
        "この注記・ルビの前に本文が存在すること、対象文字列が直前に存在していることを確認してください。\n特に、ルビは｜で範囲指定を行わない限り直前に漢字を期待します。"
    )
)]
pub struct BackRefFailed {
    #[source_code]
    pub source_code: String,
    #[label("この注記でエラーが発生しています。")]
    pub failed_note: Span,
}

#[derive(Error, Debug, Diagnostic)]
#[error("ルビデリミタの使い方が不正です")]
#[diagnostic(
    code(aozora_rs::invalid_ruby_delimiter_usage),
    url(docsrs),
    help("ルビデリミタ（｜）とルビの間に本文以外を含めることはできません。")
)]
pub struct InvalidRubyDelimiterUsage {
    #[source_code]
    pub source_code: String,
    #[label("この注記でエラーが発生しています。")]
    pub failed_note: Span,
}
