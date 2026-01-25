#[derive(Debug, Clone, Copy)]
pub struct Odoriji {
    pub has_dakuten: bool,
}

impl std::fmt::Display for Odoriji {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}〵", if self.has_dakuten { "〴" } else { "〳" })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BlockIndent {
    pub level: usize,
}

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

impl std::fmt::Display for BotenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}傍点",
            match self {
                Self::Circle => "白丸",
                Self::CircleFilled => "丸",
                Self::Crossing => "ばつ",
                Self::DoubleCircle => "二重丸",
                Self::Hebinome => "蛇の目",
                Self::Sesame => "白ゴマ",
                Self::Triangle => "白三角",
                Self::TriangleFilled => "黒三角",
            }
        )
    }
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

impl std::fmt::Display for BosenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Chain => "鎖線",
                Self::Dashed => "破線",
                Self::Double => "二重傍線",
                Self::Plain => "傍線",
                Self::Wavy => "波線",
            }
        )
    }
}

#[derive(Debug)]
pub enum Deco<'s> {
    Bold,
    Italic,
    Ruby(&'s str),
    Bosen(BosenKind),
    Boten(BotenKind),
    Indent(usize),
    Hanging((usize, usize)),
    Grounded,
    LowFlying(usize),
    AHead,
    BHead,
    CHead,
    HinV,
    Mama,
}

impl std::fmt::Display for Deco<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}]",
            match self {
                Self::Bold => "太字".to_string(),
                Self::Italic => "斜体".to_string(),
                Self::Ruby(r) => format!("ルビ「{}」", r),
                Self::Bosen(b) => b.to_string(),
                Self::Boten(b) => b.to_string(),
                Self::Indent(i) => format!("{}字下げ", i),
                Self::Hanging(h) => format!("{}字下げ、折り返して{}字下げ", h.0, h.1),
                Self::Grounded => "地付き".to_string(),
                Self::LowFlying(l) => format!("{}字寄せ", l),
                Self::AHead => "大見出し".to_string(),
                Self::BHead => "中見出し".to_string(),
                Self::CHead => "小見出し".to_string(),
                Self::HinV => "縦中横".to_string(),
                Self::Mama => "ママ".to_string(),
            }
        )
    }
}

#[derive(Debug, Clone)]
pub struct Figure<'s> {
    pub path: &'s str,
    pub caption: &'s str,
    pub size: Option<(usize, usize)>,
}

impl std::fmt::Display for Figure<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}（{}{}）入る",
            self.caption,
            self.path,
            if let Some((h, v)) = self.size {
                format!("縦{}×横{}", h, v)
            } else {
                "".to_string()
            }
        )
    }
}
