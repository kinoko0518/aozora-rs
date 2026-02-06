use crate::prelude::*;
use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
#[error("注記が閉じられていません")]
#[diagnostic(
    code(aozora_rs::unclosed_inline_note),
    help("この注記は行末で閉じられる必要があります。行を超えて注記を適用しようとしていませんか？")
)]
pub struct UnclosedInlineNote {
    #[source_code]
    pub source_code: String,
    #[label("この領域を閉じてください")]
    pub unclosed_area: Span,
}

#[derive(Error, Debug, Diagnostic)]
#[error("前方参照に失敗しました")]
#[diagnostic(
    code(aozora_rs::backref_failed),
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
    #[label("この領域でエラーが発生しています。")]
    pub failed_note: Span,
}

#[derive(Error, Debug, Diagnostic)]
#[error("タグの交差は許可されていません")]
#[diagnostic(
    code(aozora_rs::clossing_tag),
    url(docsrs),
    help("たとえば［＃A開始］［＃B開始］［＃A終了］［＃B終了］のような構造です")
)]
pub struct CrossingNote {
    #[source_code]
    pub source_code: String,
    #[label("この範囲でエラーが発生しています。")]
    pub range: Span,
}

#[derive(Error, Debug, Diagnostic)]
#[error("この終了注記はどこにも対応していません")]
#[diagnostic(
    code(aozora_rs::isolated_end_note),
    url(docsrs),
    help("この注記は［＃開始］［＃終了］の形で使用してください")
)]
pub struct IsolatedEndNote {
    #[source_code]
    pub source_code: String,
    #[label("この注記でエラーが発生しています")]
    pub range: Span,
}
