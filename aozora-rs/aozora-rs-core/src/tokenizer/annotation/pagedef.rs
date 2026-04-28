use winnow::{Parser, combinator::alt};

use crate::{Input, PageDef, WinnowError};

pub fn pagedef(input: &mut Input) -> Result<PageDef, WinnowError> {
    alt(("ページの左右中央".value(PageDef::VHCentre),)).parse_next(input)
}
