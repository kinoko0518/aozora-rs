use winnow::{Parser, combinator::alt, error::ContextError};

use crate::Input;

#[derive(Debug, Clone, Copy)]
pub enum Single {
    PageBreak,
    RectoBreak,
    SpreadBreak,
    ColumnBreak,
}

pub fn single(input: &mut Input) -> Result<Single, ContextError> {
    alt((
        "改ページ".value(Single::PageBreak),
        "改丁".value(Single::ColumnBreak),
        "改段".value(Single::ColumnBreak),
        "改見開き".value(Single::SpreadBreak),
    ))
    .parse_next(input)
}
