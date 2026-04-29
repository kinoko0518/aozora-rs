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

impl From<WinnowError> for AozoraError {
    fn from(_val: WinnowError) -> Self {
        AozoraError::TokenizeError
    }
}

impl From<AozoraEpubError> for AozoraError {
    fn from(val: AozoraEpubError) -> Self {
        AozoraError::Epub(val)
    }
}

impl From<MetaError> for AozoraError {
    fn from(val: MetaError) -> Self {
        AozoraError::Meta(val)
    }
}

impl From<AozoraZipError> for AozoraError {
    fn from(val: AozoraZipError) -> Self {
        AozoraError::Zip(val)
    }
}

impl From<std::io::Error> for AozoraError {
    fn from(val: std::io::Error) -> Self {
        AozoraError::IoError(val)
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

impl From<ScopenizeError> for AozoraWarning {
    fn from(val: ScopenizeError) -> Self {
        AozoraWarning::Scopenize(val)
    }
}

impl From<RetokenizeError> for AozoraWarning {
    fn from(val: RetokenizeError) -> Self {
        AozoraWarning::Retokenize(val)
    }
}

impl From<EpubWarning> for AozoraWarning {
    fn from(val: EpubWarning) -> Self {
        AozoraWarning::Epub(val)
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
