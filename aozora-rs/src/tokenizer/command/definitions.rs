//! スタイルで汎用的に使用される定義を行います。

use winnow::{Parser, combinator::alt, error::ContextError};

use crate::Input;
/// 圏点の見た目のEnumです。青空文庫書式における圏点の扱いについては以下のURLを参照してください。
/// 文字色によって変わる「白…」「黒…」という呼び方はここではFilledに呼び変えています。
///
/// https://www.aozora.gr.jp/annotation/emphasis.html#boten_chuki
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BotenKind {
    /// 「白ゴマ傍点」に対応
    Sesame,
    /// 「白丸傍点」に対応
    Circle,
    /// 「丸傍点」に対応
    CircleFilled,
    /// 「白三角傍点」に対応
    Triangle,
    /// 「黒三角傍点」に対応
    TriangleFilled,
    /// 「二重丸傍点」に対応
    DoubleCircle,
    /// 「蛇の目傍点」に対応
    Hebinome,
    /// 「ばつ傍点」に対応
    Crossing,
}

/// 傍線の種類のEnumです。青空文庫書式における傍線の扱いについては以下のURLを参照してください。
///
/// https://www.aozora.gr.jp/annotation/emphasis.html#bosen_chuki
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BosenKind {
    /// 「傍線」に対応
    Plain,
    /// 「二重傍線」に対応
    Double,
    /// 「鎖線」に対応
    Chain,
    /// 「破線」に対応
    Dashed,
    /// 「波線」に対応
    Wavy,
}

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
