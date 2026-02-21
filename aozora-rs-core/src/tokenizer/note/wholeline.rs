use winnow::{
    Parser,
    combinator::{alt, opt},
    error::ContextError,
};

use crate::{nihongo::japanese_num, *};

#[derive(Debug, Clone)]
pub enum WholeLine {
    /// 「N字下げ」に対応
    Indent(usize),
    /// 「地付き」に対応
    Grounded,
    /// 「地からN字寄せ」に対応
    LowFlying(usize),
    /// 「ページの左右中央」に対応
    VHCentre,
}

impl WholeLine {
    pub fn into_deco<'s>(self) -> Deco<'s> {
        match self {
            Self::Indent(n) => Deco::Indent(n),
            Self::Grounded => Deco::Grounded,
            Self::LowFlying(n) => Deco::LowFlying(n),
            Self::VHCentre => Deco::VHCentre,
        }
    }
}

pub fn wholeline(input: &mut Input) -> Result<WholeLine, ContextError> {
    alt((
        (opt("天から"), japanese_num, "字下げ").map(|(_, n, _)| WholeLine::Indent(n)),
        "地付き".value(WholeLine::Grounded),
        ("地から", japanese_num, "字上げ").map(|(_, n, _)| WholeLine::LowFlying(n)),
        "ページの左右中央".value(WholeLine::VHCentre),
    ))
    .parse_next(input)
}
