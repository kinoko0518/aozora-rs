use crate::GaijiMap;
use aozora_rs_gaiji::{Gaiji, TagSet, parse_tag, white0};
use std::collections::HashMap;
use winnow::{
    Parser,
    combinator::{alt, delimited, repeat},
    error::ContextError,
    token::any,
};

fn parse_gaiji(input: &mut &str) -> Result<Gaiji, ContextError> {
    (
        any,
        white0,
        delimited("※［＃", parse_tag("、ページ数-行数".void()), "］"),
    )
        .map(|(kanji, _, tag): (char, _, TagSet)| Gaiji {
            kanji: kanji,
            tag: tag
                .tag
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect::<String>(),
        })
        .parse_next(input)
}

pub fn extract_gaiji_entries(text: &mut &str) -> Result<GaijiMap, ContextError> {
    repeat(
        1..,
        alt((
            parse_gaiji.map(|s| Some(s)),
            any.value(Option::<Gaiji>::None),
        )),
    )
    .fold(
        || HashMap::new(),
        |mut acc, e| {
            if let Some(e) = e {
                acc.insert(e.tag, e.kanji);
            }
            acc
        },
    )
    .parse_next(text)
}
