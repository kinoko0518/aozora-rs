use std::char::from_u32;

use winnow::{Parser, combinator::alt, error::ContextError, token::take_while};

use crate::prelude::*;

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
    alt((fw_digit, hw_digit)).parse_next(input)
}
