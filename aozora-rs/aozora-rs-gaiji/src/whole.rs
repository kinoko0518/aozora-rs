use std::borrow::Cow;

use winnow::Parser;
use winnow::combinator::{alt, delimited, opt, repeat};
use winnow::token::{rest, take_until};

type WinnowError = ();

use crate::gaiji_to_char;

const GAIJI_BEGIN: &str = "※［＃";

enum GaijiOrStr<'s> {
    Gaiji(&'s str),
    Str(&'s str),
    Odoriji(bool),
}

impl<'s> GaijiOrStr<'s> {
    fn to_cow(&self) -> Result<Cow<'s, str>, &'s str> {
        match self {
            GaijiOrStr::Gaiji(g) => {
                let mut str = *g;
                Ok(Cow::Owned(gaiji_to_char(&mut str).ok_or(str)?.to_string()))
            }
            GaijiOrStr::Str(s) => Ok(Cow::Borrowed(s)),
            Self::Odoriji(b) => Ok(Cow::Borrowed(if *b { "〳〵" } else { "〴〵" })),
        }
    }
}

fn parse_text<'a>(input: &mut &'a str) -> Result<GaijiOrStr<'a>, WinnowError> {
    alt((
        take_until(1.., GAIJI_BEGIN),
        rest.verify(|s: &str| !s.is_empty()),
    ))
    .map(GaijiOrStr::Str)
    .parse_next(input)
}

fn parse_gaiji<'a>(input: &mut &'a str) -> Result<GaijiOrStr<'a>, WinnowError> {
    delimited(GAIJI_BEGIN, take_until(0.., "］"), "］")
        .map(GaijiOrStr::Gaiji)
        .parse_next(input)
}

fn parse_odoriji<'a>(input: &mut &'a str) -> Result<GaijiOrStr<'a>, WinnowError> {
    ("／", opt('″'), "＼")
        .map(|(_, dakuten, _)| dakuten.is_some())
        .map(GaijiOrStr::Odoriji)
        .parse_next(input)
}

pub fn utf8tify_all_gaiji<'s>(input: &'s str) -> (Cow<'s, str>, Vec<&'s str>) {
    let mut input = input;
    let result: Vec<GaijiOrStr> = repeat(0.., alt((parse_gaiji, parse_text, parse_odoriji)))
        .parse_next(&mut input)
        .unwrap();
    if result.is_empty() {
        (Cow::Borrowed(input), vec![])
    } else if result.len() == 1 {
        match result.into_iter().next().unwrap().to_cow() {
            Ok(o) => (o, vec![]),
            Err(e) => ("〓".into(), vec![e]),
        }
    } else {
        let mut err = vec![];
        (
            Cow::Owned(
                result
                    .into_iter()
                    .fold(String::new(), |mut acc: String, r| {
                        let converted = match r.to_cow() {
                            Ok(o) => o,
                            Err(e) => {
                                err.push(e);
                                "〓".into()
                            }
                        };
                        acc.push_str(converted.as_ref());
                        acc
                    }),
            ),
            err,
        )
    }
}
