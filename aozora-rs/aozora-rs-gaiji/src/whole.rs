use std::borrow::Cow;

use winnow::combinator::{alt, delimited, repeat};
use winnow::token::{rest, take_until};
use winnow::{Parser, error::ContextError};

use crate::gaiji_to_char;

const GAIJI_BEGIN: &str = "※［＃";

enum GaijiOrStr<'s> {
    Gaiji(&'s str),
    Str(&'s str),
}

impl<'s> GaijiOrStr<'s> {
    fn to_cow(&self) -> Cow<'s, str> {
        match self {
            GaijiOrStr::Gaiji(g) => {
                let mut str = *g;
                Cow::Owned(
                    gaiji_to_char(&mut str)
                        .unwrap_or(Cow::Borrowed("〓"))
                        .to_string(),
                )
            }
            GaijiOrStr::Str(s) => Cow::Borrowed(s),
        }
    }
}

fn parse_text<'a>(input: &mut &'a str) -> Result<GaijiOrStr<'a>, ContextError> {
    alt((
        take_until(1.., GAIJI_BEGIN),
        rest.verify(|s: &str| !s.is_empty()),
    ))
    .map(GaijiOrStr::Str)
    .parse_next(input)
}

fn parse_gaiji<'a>(input: &mut &'a str) -> Result<GaijiOrStr<'a>, ContextError> {
    delimited(GAIJI_BEGIN, take_until(0.., "］"), "］")
        .map(GaijiOrStr::Gaiji)
        .parse_next(input)
}

pub fn utf8tify_all_gaiji<'s>(input: &'s str) -> Cow<'s, str> {
    let mut input = input;
    let result: Vec<GaijiOrStr> = repeat(0.., alt((parse_gaiji, parse_text)))
        .parse_next(&mut input)
        .unwrap();
    if result.is_empty() {
        Cow::Borrowed(input)
    } else if result.len() == 1 {
        result.into_iter().next().unwrap().to_cow()
    } else {
        Cow::Owned(
            result
                .into_iter()
                .fold(String::new(), |mut acc: String, r| {
                    acc.push_str(r.to_cow().as_ref());
                    acc
                }),
        )
    }
}
