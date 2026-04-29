//! メタデータを扱うためのモジュールです。
//!
//! タイトル、著者、【テキスト中に現れる記号について】といったファイルの先頭に記述される情報がメタデータに対応します。

use winnow::{Parser, combinator::delimited, error::ContextError, token::take_until};

/// 青空文庫で記述されたテキストのメタデータをまとめた型です。
#[derive(Debug, Clone)]
pub struct AozoraMeta<'s> {
    /// タイトルです。
    pub title: &'s str,
    /// 著者です。
    pub author: &'s str,
}

/// メタデータ取得中に発生しうるエラーの直和です。
#[derive(Debug)]
pub enum MetaError {
    /// タイトルの記述が見つからなかったときのエラーです。
    NoTitleFound,
    /// 著者の記述が見つからなかったときのエラーです。
    NoAuthorFound,
}

impl std::fmt::Display for MetaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::NoAuthorFound => "著者の表記形式が無効です",
                Self::NoTitleFound => "タイトルの表記形式が無効です",
            }
        )
    }
}

/// タイトル、著者、【テキスト中に現れる記号について】といったファイルの先頭に記述される特別な情報を解析し、AozoraMetaに纏めます。
///
/// パース成功後、`input` はメタデータ部分が消費された本文の先頭を指します。
pub fn parse_meta<'s>(input: &mut &'s str) -> Result<AozoraMeta<'s>, MetaError> {
    // タイトルをパース
    let title: &str = (take_until::<_, _, ContextError>(1.., "\n"), '\n')
        .map(|(s, _): (&str, char)| s.trim())
        .parse_next(input)
        .ok()
        .ok_or(MetaError::NoTitleFound)?;
    // 著者をパース
    let author: &str = (take_until::<_, _, ContextError>(1.., "\n"), '\n')
        .map(|(s, _): (&str, char)| s.trim())
        .parse_next(input)
        .ok()
        .ok_or(MetaError::NoAuthorFound)?;
    // 【テキスト中に現れる記号について】をパース
    let about_symbol = "-------------------------------------------------------";
    let _: Result<(), _> = take_until::<_, _, ContextError>(0.., about_symbol)
        .void()
        .parse_next(input);
    let _: Result<(), ContextError> =
        delimited(about_symbol, take_until(0.., about_symbol), about_symbol)
            .void()
            .parse_next(input);
    Ok(AozoraMeta { title, author })
}
