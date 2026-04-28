#![doc = include_str!("../../../docs/note/wholeline.md")]

use winnow::{
    Parser,
    combinator::{alt, opt},
};

use crate::{nihongo::japanese_num, *};

/// 行頭型注記の直和です。
///
/// 行の頭に置き、行末まで影響を及ぼす注記が分類されます。
#[derive(Debug, Clone, PartialEq, Eq)]
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

impl Into<Deco<'static>> for WholeLine {
    fn into(self) -> Deco<'static> {
        match self {
            Self::Indent(n) => Deco::Indent(n),
            Self::Grounded => Deco::Grounded,
            Self::LowFlying(n) => Deco::LowFlying(n),
            Self::VHCentre => Deco::VHCentre,
        }
    }
}

impl Into<AozoraTokenKind<'static>> for WholeLine {
    fn into(self) -> AozoraTokenKind<'static> {
        AozoraTokenKind::Annotation(Annotation::WholeLine(self))
    }
}

pub(crate) fn wholeline(input: &mut Input) -> Result<WholeLine, WinnowError> {
    alt((
        (opt("天から"), japanese_num, "字下げ").map(|(_, n, _)| WholeLine::Indent(n)),
        "地付き".value(WholeLine::Grounded),
        ("地から", japanese_num, "字上げ").map(|(_, n, _)| WholeLine::LowFlying(n)),
        "ページの左右中央".value(WholeLine::VHCentre),
    ))
    .parse_next(input)
}
