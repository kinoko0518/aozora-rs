use std::{borrow::Cow, collections::HashMap};

use winnow::{
    Parser,
    ascii::{digit1, hex_digit1, space0},
    combinator::{alt, eof, opt, repeat, repeat_till},
    error::ContextError,
    token::any,
};

pub type Menkuten = (u8, u8, u8);

#[derive(Debug, PartialEq)]
pub struct GaijiChuki<'s> {
    pub tag: &'s str,
    pub sjis: Option<Menkuten>,
    pub unicode: Option<String>,
}

impl GaijiChuki<'_> {
    pub fn to_cow<'s>(
        self,
        gaiji_to_char: &'s HashMap<String, String>,
        menkuten: &'s HashMap<Menkuten, String>,
    ) -> Option<Cow<'s, str>> {
        self.unicode
            .map(Cow::Owned)
            .or(self
                .sjis
                .and_then(|code| menkuten.get(&code).map(|got| Cow::Borrowed(got.as_str()))))
            .or(gaiji_to_char
                .get(self.tag)
                .map(|c| Cow::Borrowed(c.as_str())))
    }
}

/// 第 1水準 1-45-14のような文字列にマッチし、面、句、点を抽出する
pub fn shift_jis(input: &mut &str) -> Result<Menkuten, ContextError> {
    (
        opt(("第", space0, digit1, "水準", space0)).void(),
        digit1,
        '-',
        digit1,
        '-',
        digit1,
    )
        .verify_map(
            |(_, men, _, ku, _, ten): (_, &str, _, &str, _, &str)| -> Option<Menkuten> {
                Some((
                    men.parse::<u8>().ok()?,
                    ku.parse::<u8>().ok()?,
                    ten.parse::<u8>().ok()?,
                ))
            },
        )
        .parse_next(input)
}

/// 16進数で表現された数字にマッチし、charに変換して値を返す
fn hex_digit_as_char(input: &mut &str) -> Result<char, ContextError> {
    hex_digit1
        .verify_map(|digit| u32::from_str_radix(digit, 16).ok())
        .verify_map(char::from_u32)
        .parse_next(input)
}

/// U+12EFやU+12EF+34CDのような文字列にマッチし、それぞれの4桁の数字をUnicodeと解釈してStringに集約する
pub fn unicode(input: &mut &str) -> Result<String, ContextError> {
    let code = ('+', hex_digit_as_char).map(|(_, code): (_, char)| code);
    ("U", repeat(0.., code))
        .map(|(_, multi): (_, String)| multi)
        .parse_next(input)
}

fn tag_limiter(
    input: &mut &str,
) -> Result<(Option<String>, Option<Menkuten>), ContextError> {
    (
        opt(('、', unicode)),
        opt(('、', shift_jis)),
        opt(alt((
            "、ページ数-行数".void(),
            (
                '、',
                digit1,
                opt(('-', alt(('上', '中', '下')))),
                '-',
                digit1,
            )
                .void(),
        ))),
    )
        .map(|(unicode, sjis, _)| {
            (
                unicode.map(|(_, unicode)| unicode),
                sjis.map(|(_, sjis)| sjis),
            )
        })
        .parse_next(input)
}

pub fn parse_tag<'s>(input: &mut &'s str) -> Result<GaijiChuki<'s>, ContextError> {
    (
        opt("「※」は"),
        repeat_till(
            1..,
            any.void(),
            alt((
                winnow::combinator::peek(tag_limiter.take().verify(|s: &str| !s.is_empty())).void(),
                eof.void(),
            )),
        )
        .map(|_: ((), _)| ())
        .take(),
        tag_limiter,
    )
        .map(|(_, tag, (unicode, sjis))| -> GaijiChuki { GaijiChuki { tag, sjis, unicode } })
        .parse_next(input)
}
