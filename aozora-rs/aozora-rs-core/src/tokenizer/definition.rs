use crate::tokenizer::*;
use crate::*;
use winnow::{Parser, combinator::alt};

/// トークンの種類、およびその固有情報構造の直和です。
#[derive(Debug, Clone)]
pub enum AozoraTokenKind<'s> {
    /// 注記（［＃……］）に対応します。
    Note(Note<'s>),
    /// ルビ（《……》）に対応します。
    Ruby(&'s str),
    /// ルビ区切り（｜）に対応します。
    RubyDelimiter,
    /// 本文となるテキストを切り出したものです。
    Text(&'s str),
    /// 改行（\n）に対応します。
    Br,
}

/// トークンを表現する構造体です。
///
/// 単にトークンごとの固有データである[`AozoraTokenKind`]とは明確に別の構造体です。
#[derive(Debug, Clone)]
pub struct Tokenized<'s> {
    /// トークンごとの固有データです。
    pub kind: AozoraTokenKind<'s>,
    /// トークンの位置情報です。
    pub span: Span,
}

pub(crate) fn boten(input: &mut Input) -> Result<BotenKind, WinnowError> {
    (
        alt((
            "白ゴマ".value(BotenKind::Sesame),
            "白丸".value(BotenKind::Circle),
            "丸".value(BotenKind::CircleFilled),
            "白三角".value(BotenKind::Triangle),
            "黒三角".value(BotenKind::TriangleFilled),
            "二重丸".value(BotenKind::DoubleCircle),
            "蛇の目".value(BotenKind::Hebinome),
            "ばつ".value(BotenKind::Crossing),
            "".value(BotenKind::Sesame),
        )),
        alt(("傍点", "圏点")).void(),
    )
        .map(|(bk, _)| bk)
        .parse_next(input)
}

pub(crate) fn bosen(input: &mut Input) -> Result<BosenKind, WinnowError> {
    alt((
        "傍線".value(BosenKind::Plain),
        "二重傍線".value(BosenKind::Double),
        "鎖線".value(BosenKind::Chain),
        "破線".value(BosenKind::Dashed),
        "波線".value(BosenKind::Wavy),
    ))
    .parse_next(input)
}
