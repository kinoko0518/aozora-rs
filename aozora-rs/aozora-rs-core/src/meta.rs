use miette::Diagnostic;
use thiserror::Error;
use winnow::{Parser, combinator::delimited, error::ContextError, token::take_until};

pub struct AozoraMeta<'s> {
    pub title: &'s str,
    pub author: &'s str,
}

#[derive(Error, Debug, Diagnostic)]
#[error("タイトルが必要です")]
#[diagnostic(
    code(aozora_rs::no_title_found),
    help("一行目はタイトルとして扱われます")
)]
struct NoTitleFound;

#[derive(Error, Debug, Diagnostic)]
#[error("著者名が必要です")]
#[diagnostic(
    code(aozora_rs::no_author_found),
    help("二行目は著者名として扱われます")
)]
struct NoAuthorFound;

/// タイトル、著者、【テキスト中に現れる記号について】といったファイルの先頭に記述される特別な情報を解析し、AozoraMetaに纏めます。
///
/// パース成功後、`input` はメタデータ部分が消費された本文の先頭を指します。
pub fn parse_meta<'s>(input: &mut &'s str) -> Result<AozoraMeta<'s>, miette::Error> {
    // タイトルをパース
    let title: &str = (take_until::<_, _, ContextError>(1.., "\n"), '\n')
        .map(|(s, _): (&str, char)| s.trim())
        .parse_next(input)
        .ok()
        .ok_or::<miette::Error>(NoTitleFound.into())?;
    // 著者をパース
    let author: &str = (take_until::<_, _, ContextError>(1.., "\n"), '\n')
        .map(|(s, _): (&str, char)| s.trim())
        .parse_next(input)
        .ok()
        .ok_or::<miette::Error>(NoAuthorFound.into())?;
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
