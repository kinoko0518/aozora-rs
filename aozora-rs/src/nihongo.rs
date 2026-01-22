use std::char::from_u32;

use winnow::{
    Parser,
    combinator::{alt, opt},
    error::ContextError,
    token::take_while,
};

use crate::Input;

pub fn is_hiragana(c: char) -> bool {
    matches!(c, 'ぁ'..='ゖ')
}

pub fn is_katakana(c: char) -> bool {
    matches!(c, 'ァ'..='ー')
}

pub fn is_kanji(c: char) -> bool {
    match c {
        '々' | '〆' | '〇' | 'ヶ' | '仝' => true,
        c if ('\u{4E00}'..='\u{9FFF}').contains(&c) => true,
        c if ('\u{3400}'..='\u{4DBF}').contains(&c) => true,
        c if ('\u{F900}'..='\u{FAFF}').contains(&c) => true,
        c if ('\u{20000}'..='\u{2A6DF}').contains(&c) => true,
        _ => false,
    }
}

pub fn single_kanji_num(input: &mut Input) -> Result<usize, ContextError> {
    alt((
        alt(('零', '〇')).value(0),
        alt(('一', '壱', '壹', '弌')).value(1),
        alt(('二', '弐', '貳', '貮', '弍')).value(2),
        alt(('三', '参', '參', '弎')).value(3),
        alt(('四', '肆', '亖')).value(4),
        alt(('五', '伍')).value(5),
        alt(('六', '陸')).value(6),
        alt(('七', '漆', '質', '柒')).value(7),
        alt(('八', '捌')).value(8),
        alt(('九', '玖')).value(9),
    ))
    .parse_next(input)
}

fn kanji_digit<'s>(kanji: char, corr: usize) -> impl Parser<Input<'s>, usize, ContextError> {
    (opt(single_kanji_num), kanji).map(move |(u, _)| u.unwrap_or(1) * corr)
}

fn kanji_optional<'s>(kanji: char, corr: usize) -> impl Parser<Input<'s>, usize, ContextError> {
    opt(kanji_digit(kanji, corr)).map(|o| o.unwrap_or(0))
}

fn kanji_num_per_kilo(input: &mut Input) -> Result<usize, ContextError> {
    (
        kanji_optional('千', 1000),
        kanji_optional('百', 100),
        kanji_optional('十', 10),
        opt(single_kanji_num),
    )
        .map(|(a, b, c, d)| a + b + c + d.unwrap_or(0))
        .parse_next(input)
}

fn kanji_num(input: &mut Input) -> Result<usize, ContextError> {
    (
        opt((kanji_num_per_kilo, kanji_optional('億', 100000000)).map(|(u, o)| u * o)),
        opt((kanji_num_per_kilo, kanji_optional('万', 10000)).map(|(u, o)| u * o)),
        kanji_num_per_kilo,
    )
        .map(|(a, b, c)| a.unwrap_or(0) + b.unwrap_or(0) + c)
        .parse_next(input)
}

fn fw_digit_to_hw(original: char) -> Option<char> {
    (('０' as u32)..=('９' as u32))
        .contains(&(original as u32))
        .then_some(original as u32 - '０' as u32 + '0' as u32)
        .and_then(|u| from_u32(u))
}

fn fw_digit(input: &mut Input) -> Result<usize, ContextError> {
    take_while(1.., |c| matches!(c, '０'..'９'))
        .map(|s: &str| {
            s.chars()
                .map(|c| fw_digit_to_hw(c).unwrap())
                .collect::<String>()
        })
        .map(|s: String| s.parse::<usize>().unwrap())
        .parse_next(input)
}

fn hw_digit(input: &mut Input) -> Result<usize, ContextError> {
    take_while(1.., |c| matches!(c, '0'..'9'))
        .map(|s: &str| s.parse::<usize>().unwrap())
        .parse_next(input)
}

pub fn japanese_num(input: &mut Input) -> Result<usize, ContextError> {
    alt((fw_digit, hw_digit, kanji_num)).parse_next(input)
}
