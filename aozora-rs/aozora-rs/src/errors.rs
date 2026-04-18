use aozora_rs_core::{MetaError, RetokenizeError, ScopenizeError, WinnowError};
use aozora_rs_epub::{AozoraEpubError, EpubWarning};
use aozora_rs_zip::AozoraZipError;

/// aozora-rsで発生しうるエラーをまとめた列挙型です。
#[derive(Debug)]
pub enum AozoraError {
    /// トークナイズに失敗したことを表すエラーです。
    ///
    /// 原理的に発生しないはずなので、発生した場合は可能な場合は発生した入力がわかる形で、
    /// お手数ですがGitHubにissueを立てていただければ対応します。
    TokenizeError,
    /// EPUB構築中に発生したエラーです。
    Epub(AozoraEpubError),
    /// メタデータ解析中に発生したエラーです。
    Meta(MetaError),
    /// 青空文庫で配布されている形式の.zipを呼んでいるときに発生したエラーです。
    Zip(AozoraZipError),
    /// 入出力時に発生したエラーです。
    IoError(std::io::Error),
}

impl Into<AozoraError> for WinnowError {
    fn into(self) -> AozoraError {
        AozoraError::TokenizeError
    }
}

impl Into<AozoraError> for AozoraEpubError {
    fn into(self) -> AozoraError {
        AozoraError::Epub(self)
    }
}

impl Into<AozoraError> for MetaError {
    fn into(self) -> AozoraError {
        AozoraError::Meta(self)
    }
}

impl Into<AozoraError> for AozoraZipError {
    fn into(self) -> AozoraError {
        AozoraError::Zip(self)
    }
}

impl Into<AozoraError> for std::io::Error {
    fn into(self) -> AozoraError {
        AozoraError::IoError(self)
    }
}

impl std::fmt::Display for AozoraError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err: String = match self {
            Self::Epub(d) => d.to_string(),
            Self::IoError(i) => i.to_string(),
            Self::Meta(m) => m.to_string(),
            Self::TokenizeError => "トークン化に失敗しました".into(),
            Self::Zip(z) => z.to_string(),
        };
        writeln!(f, "{}", err)
    }
}

impl std::error::Error for AozoraError {}

/// aozora-rsが発生させうるWarningの列挙型です。
pub enum AozoraWarning {
    /// 注記やルビの影響範囲の確定中に発生したエラーです。
    Scopenize(ScopenizeError),
    /// 中間表現の生成中に発生したエラーです。
    Retokenize(RetokenizeError),
    /// EPUBの構築中に発生したWarningです。
    Epub(EpubWarning),
}

impl Into<AozoraWarning> for ScopenizeError {
    fn into(self) -> AozoraWarning {
        AozoraWarning::Scopenize(self)
    }
}

impl Into<AozoraWarning> for RetokenizeError {
    fn into(self) -> AozoraWarning {
        AozoraWarning::Retokenize(self)
    }
}

impl Into<AozoraWarning> for EpubWarning {
    fn into(self) -> AozoraWarning {
        AozoraWarning::Epub(self)
    }
}

impl AozoraWarning {
    /// Warningの表示を行います。
    ///
    /// どこでエラーが発生したのかを指し示すため、原文のメタデータを除いた部分が必要です。
    pub fn display(&self, original: &str) -> String {
        match self {
            AozoraWarning::Retokenize(r) => r.to_string(),
            AozoraWarning::Scopenize(s) => s.display(original),
            AozoraWarning::Epub(z) => z.to_string(),
        }
    }
}
