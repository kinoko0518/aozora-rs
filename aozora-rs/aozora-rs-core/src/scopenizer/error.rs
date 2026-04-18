use crate::*;

#[derive(Debug)]
pub enum ScopenizeError {
    UnclosedInlineNote(Span),
    BackRefFailed(Span),
    InvalidRubyDelimiterUsage(Span),
    CrossingNote(Span),
    IsolatedEndNote(Span),
}

impl Default for ScopenizeError {
    fn default() -> Self {
        ScopenizeError::BackRefFailed(0..0)
    }
}

impl ScopenizeError {
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
