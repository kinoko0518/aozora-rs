//! スタイルで汎用的に使用される定義を行います。
/// 圏点の見た目のEnumです。青空文庫書式における圏点の扱いについては以下のURLを参照してください。
/// 文字色によって変わる「白…」「黒…」という呼び方はここではFilledに呼び変えています。
///
/// https://www.aozora.gr.jp/annotation/emphasis.html#boten_chuki
#[derive(PartialEq, Eq)]
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
    /// 「二十丸傍点」に対応
    DoubleCircle,
    /// 「蛇の目傍点」に対応
    Hebinome,
    /// 「ばつ傍点」に対応
    Crossing,
}

/// 傍線の種類のEnumです。青空文庫書式における傍線の扱いについては以下のURLを参照してください。
///
/// https://www.aozora.gr.jp/annotation/emphasis.html#bosen_chuki
#[derive(PartialEq, Eq)]
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
