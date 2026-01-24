//! スタイルで汎用的に使用される定義を行います。

use winnow::{Parser, combinator::alt, error::ContextError};

use crate::prelude::*;

pub fn boten(input: &mut Input) -> Result<BotenKind, ContextError> {
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

pub fn bosen(input: &mut Input) -> Result<BosenKind, ContextError> {
    alt((
        "傍線".value(BosenKind::Plain),
        "二重傍線".value(BosenKind::Double),
        "鎖線".value(BosenKind::Chain),
        "破線".value(BosenKind::Dashed),
        "波線".value(BosenKind::Wavy),
    ))
    .parse_next(input)
}
