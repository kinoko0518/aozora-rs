use miette::Diagnostic;
use thiserror::Error;
use winnow::{
    LocatingSlice, Parser, combinator::delimited, error::ContextError, token::take_until,
};

use crate::{
    prelude::{AozoraToken, Retokenized, retokenize, scopenize},
    tokenizer::prelude::tokenize_nometa,
};

mod deco;
mod nihongo;
mod retokenizer;
mod scopenizer;
mod tokenizer;

pub mod prelude {
    use winnow::LocatingSlice;

    pub type Input<'s> = LocatingSlice<&'s str>;
    pub type Span = std::ops::Range<usize>;

    pub use crate::deco::*;

    pub use crate::scopenizer::definition::{Break, FlatToken, Scope};
    pub use crate::scopenizer::prelude::scopenize;

    pub use crate::tokenizer::definition::{AozoraToken, AozoraTokenKind};
    pub use crate::tokenizer::prelude::{Note, tokenize_nometa};

    pub use crate::retokenizer::prelude::*;

    pub use crate::*;
}

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

#[derive(Error, Debug, Diagnostic)]
#[error("トークン化に失敗しました")]
#[diagnostic(
    code(aozora_rs::no_author_found),
    help("原理的にトークン化は失敗しません。お手間をかけますが、バグ報告を頂ければ幸いです。")
)]
struct TokenizeFailed;

pub fn tokenize<'s>(
    input: &'s str,
) -> Result<(AozoraMeta<'s>, Vec<AozoraToken<'s>>), miette::Error> {
    let input = &mut LocatingSlice::new(input);
    // タイトルをパース
    let title: &str = (take_until::<_, _, ContextError>(1.., "\n"), '\n')
        .map(|(s, _)| s)
        .parse_next(input)
        .ok()
        .ok_or::<miette::Error>(NoTitleFound.into())?;
    // 著者をパース
    let author: &str = (take_until::<_, _, ContextError>(1.., "\n"), '\n')
        .map(|(s, _)| s)
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
    // 本文を処理
    Ok((
        AozoraMeta { title, author },
        tokenize_nometa(input)
            .ok()
            .ok_or::<miette::Error>(TokenizeFailed.into())?,
    ))
}

pub fn parse<'s>(input: &'s str) -> AZResult<(AozoraMeta<'s>, Vec<Retokenized<'s>>)> {
    let (meta, tokens) = tokenize(input).unwrap();
    let ((scopenized, flattoken), errors) = scopenize(tokens, input).into_tuple();
    let retokenizde = retokenize(flattoken, scopenized);
    AZResultC::from(errors).finally((meta, retokenizde))
}

pub struct AZResultC {
    errors: Vec<miette::Error>,
}

impl AZResultC {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn from(errors: Vec<miette::Error>) -> Self {
        Self { errors }
    }

    pub fn push(&mut self, e: miette::Error) {
        self.errors.push(e);
    }

    pub fn finally<T>(self, result: T) -> AZResult<T> {
        AZResult {
            inside: result,
            errors: self.errors,
        }
    }
}

/// Graceful Degradationに対応するためのResult型
pub struct AZResult<T> {
    inside: T,
    errors: Vec<miette::Error>,
}

impl<T> AZResult<T> {
    pub fn unwrap(self) -> T {
        for e in self.errors {
            eprintln!("{:?}", e);
        }
        self.inside
    }

    pub fn into_tuple(self) -> (T, Vec<miette::Error>) {
        (self.inside, self.errors)
    }
}
