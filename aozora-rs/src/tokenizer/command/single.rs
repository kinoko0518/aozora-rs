use winnow::{Parser, combinator::alt, error::ContextError};

use crate::Input;

#[derive(Debug, Clone, Copy)]
pub enum Single {
    /// 「改ページ」に対応
    PageBreak,
    /// 「改丁」に対応
    RectoBreak,
    /// 「改見開き」に対応
    SpreadBreak,
    /// 「改段」に対応
    ColumnBreak,
}

pub fn single(input: &mut Input) -> Result<Single, ContextError> {
    alt((
        "改ページ".value(Single::PageBreak),
        "改丁".value(Single::RectoBreak),
        "改段".value(Single::ColumnBreak),
        "改見開き".value(Single::SpreadBreak),
    ))
    .parse_next(input)
}
