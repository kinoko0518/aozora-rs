use crate::*;

/// [`scopenize`]の過程で発生するエラーの直和です。
#[derive(Debug)]
pub enum ScopenizeError {
    /// 行内挟み込み型の注記が閉じられなかったときに発生するエラーです。
    UnclosedInlineNote(Span),
    /// 前方参照型が参照するテキストが直前のトークンに見つからなかったときに発生するエラーです。
    BackRefFailed(Span),
    /// ルビデリミタのあとにテキストトークン、ルビの順番で並んでいなかったときに発生するエラーです。
    InvalidRubyDelimiterUsage(Span),
    /// 影響範囲が交差している注記が存在するときに発生するエラーです。
    CrossingNote(Span),
    /// 開始されなかったにもかかわらず終了された注記が存在するときに発生するエラーです。
    IsolatedEndNote(Span),
}

impl Default for ScopenizeError {
    fn default() -> Self {
        ScopenizeError::BackRefFailed(0..0)
    }
}

impl ScopenizeError {
    /// `original`を受け取り、人間に親切な形でエラーを表示します。
    pub fn display(&self, original: &str) -> String {
        display_error_with_decolation(
            original,
            match self {
                ScopenizeError::BackRefFailed(s) => s,
                ScopenizeError::CrossingNote(s) => s,
                ScopenizeError::InvalidRubyDelimiterUsage(s) => s,
                ScopenizeError::IsolatedEndNote(s) => s,
                ScopenizeError::UnclosedInlineNote(s) => s,
            }
            .clone(),
            "ScopenizeError",
            match self {
                Self::BackRefFailed(_) => "前方参照に失敗しました",
                Self::CrossingNote(_) => "注記が交差しています",
                ScopenizeError::InvalidRubyDelimiterUsage(_) => "ルビの使用方法が不正です",
                ScopenizeError::IsolatedEndNote(_) => "開始注記のない終了注記が存在します",
                ScopenizeError::UnclosedInlineNote(_) => "行内注記が閉じられていません",
            },
        )
    }
}
