use crate::{GAIJI_TO_CHAR, shift_jis::JISCharactor};
use winnow::{
    Parser,
    combinator::{alt, delimited, eof, opt, peek, repeat, repeat_till},
    error::ContextError,
    token::{any, one_of, take_while},
};

pub fn hex(input: &mut &str) -> Result<u32, ContextError> {
    let hex = alt((
        one_of('0'..='9').map(|u| (u as u8) - b'0'),
        one_of('A'..='F').map(|u| (u as u8) - b'A' + 10),
        one_of('a'..='f').map(|u| (u as u8) - b'a' + 10),
    ));
    repeat(1..=6, hex)
        .map(|u: Vec<u8>| u.iter().fold(0_u32, |acc, i| (acc << 4) | (*i as u32)))
        .parse_next(input)
}

pub fn white0<'s>(input: &mut &'s str) -> Result<&'s str, ContextError> {
    take_while(0.., char::is_whitespace).parse_next(input)
}

pub fn shift_jis(input: &mut &str) -> Result<JISCharactor, ContextError> {
    (
        opt((
            delimited("第", (white0, digit1_wide_u32, white0), "水準"),
            white0,
        )
            .map(|((_, u, _), _)| u)),
        opt(white0),
        digit1_wide_u32,
        "-",
        digit1_wide_u32,
        "-",
        digit1_wide_u32,
    )
        .verify_map(|(level, _, face, _, area, _, point)| {
            JISCharactor::new(
                level.unwrap_or(2) as u16,
                face as u16,
                area as u16,
                point as u16,
            )
        })
        .parse_next(input)
}

#[derive(Debug, Clone, Copy)]
pub struct Unicode(u32);

impl Unicode {
    fn hex_to_char(hex: u32) -> Option<char> {
        if hex <= 9 {
            char::from_u32(hex + (b'0' as u32))
        } else if 10 <= hex && hex <= 15 {
            char::from_u32(hex - 10 + (b'A' as u32))
        } else {
            None
        }
    }
    pub fn to_char(&self) -> Option<char> {
        char::from_u32(self.0)
    }
    pub fn from_char(c: char) -> Self {
        Self(c as u32)
    }
    pub fn to_string(&self) -> Option<String> {
        let top = Self::hex_to_char(self.0 >> 24)?;
        let highmiddle = Self::hex_to_char(self.0 << 8 >> 24)?;
        let lowmiddle = Self::hex_to_char(self.0 << 16 >> 24)?;
        let bottom = Self::hex_to_char(self.0 << 24 >> 24)?;
        Some(format!("U+{}{}{}{}", top, highmiddle, lowmiddle, bottom))
    }
}

pub fn unicode(input: &mut &str) -> Result<Unicode, ContextError> {
    ("U", one_of(('+', '＋')), hex)
        .map(|(_, _, hex)| Unicode(hex))
        .parse_next(input)
}

pub fn digit1_wide_u32(input: &mut &str) -> Result<u32, ContextError> {
    let digits_str = take_while(1.., |c: char| c.is_digit(10)).parse_next(input)?;

    let value = digits_str
        .chars()
        .fold(0u32, |acc, c| acc * 10 + c.to_digit(10).unwrap_or(0));

    Ok(value)
}

pub fn parse_tag<'s>(
    page_and_line: impl Parser<&'s str, (), ContextError>,
) -> impl Parser<&'s str, TagSet, ContextError> {
    type TagMaybe = Option<(Option<Unicode>, Option<JISCharactor>)>;

    let end = alt((
        (
            opt(("、", unicode)),
            opt((white0, "、", white0, shift_jis)),
            opt(page_and_line),
            peek(alt(("］", eof))),
        )
            .map(|(uni, shift, _, _)| Some((uni.map(|(_, u)| u), shift.map(|(_, _, _, s)| s)))),
        eof.value(TagMaybe::None),
    ));

    repeat_till(0.., any, end).map(|(tag, found): (String, TagMaybe)| match found {
        Some(s) => TagSet {
            unicode: s.0,
            shift_jis: s.1,
            tag,
        },
        None => TagSet {
            unicode: None,
            shift_jis: None,
            tag,
        },
    })
}

#[derive(Debug, Clone)]
pub struct Gaiji {
    pub kanji: char,
    pub tag: String,
}

#[derive(Debug)]
pub struct TagSet {
    unicode: Option<Unicode>,
    shift_jis: Option<JISCharactor>,
    pub tag: String,
}

impl TagSet {
    pub fn char(&self) -> Option<char> {
        GAIJI_TO_CHAR
            .get(&self.tag)
            .map(|c| *c)
            .or_else(|| self.unicode.and_then(|u| u.to_char()))
            .or_else(|| self.shift_jis.and_then(|s| s.to_char()))
    }
}

pub fn location(input: &mut &str) -> Result<(), ContextError> {
    let upper_or_lower = (
        "-",
        alt((
            one_of(('上', '中', '下')).void(),
            (digit1_wide_u32, "段").void(),
            "本文".void(),
        )),
    );
    let maki = (digit1_wide_u32, "巻", "-");
    (
        "、",
        opt(maki),
        digit1_wide_u32,
        opt(upper_or_lower),
        "-",
        alt((
            digit1_wide_u32.void(),
            ("図", opt('の'), "キャプション").void(),
        )),
    )
        .void()
        .parse_next(input)
}
