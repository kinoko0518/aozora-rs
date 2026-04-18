use aozora_rs_core::{MetaError, RetokenizeError, ScopenizeError, WinnowError};
use aozora_rs_epub::{AozoraZipError, AozoraZipWarning};
use aozora_rs_zip::DependenciesError;

#[derive(Debug)]
pub enum AozoraError {
    TokenizeError,
    Dependencies(DependenciesError),
    Meta(MetaError),
    Zip(AozoraZipError),
    IoError(std::io::Error),
}

impl Into<AozoraError> for WinnowError {
    fn into(self) -> AozoraError {
        AozoraError::TokenizeError
    }
}

impl Into<AozoraError> for DependenciesError {
    fn into(self) -> AozoraError {
        AozoraError::Dependencies(self)
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
            Self::Dependencies(d) => d.to_string(),
            Self::IoError(i) => i.to_string(),
            Self::Meta(m) => m.to_string(),
            Self::TokenizeError => "トークン化に失敗しました".into(),
            Self::Zip(z) => z.to_string(),
        };
        writeln!(f, "{}", err)
    }
}

impl std::error::Error for AozoraError {}

pub enum AozoraWarning {
    Scopenize(ScopenizeError),
    Retokenize(RetokenizeError),
    Zip(AozoraZipWarning),
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

impl Into<AozoraWarning> for AozoraZipWarning {
    fn into(self) -> AozoraWarning {
        AozoraWarning::Zip(self)
    }
}

impl AozoraWarning {
    pub fn display(&self, original: &str) -> String {
        match self {
            AozoraWarning::Retokenize(r) => r.to_string(),
            AozoraWarning::Scopenize(s) => s.display(original),
            AozoraWarning::Zip(z) => z.to_string(),
        }
    }
}
