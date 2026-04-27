//! 装飾の型定義を行うモジュールです。

use std::borrow::Cow;

use crate::{AozoraTokenKind, Note, Single};

/// 段落（HTMLで云うところの<p>などのブロック要素）に対する字下げを表現する型です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockIndent {
    /// N字下げのNに対応します。
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

/// 青空文庫書式で扱われる装飾のEnumです。
#[derive(Debug, Clone, Default)]
pub enum Deco<'s> {
    #[default]
    /// 太字に対応します。
    Bold,
    /// 斜体に対応します。
    Italic,
    /// ルビに対応します。
    Ruby(&'s str),
    /// 傍線に対応します。
    Bosen(BosenKind),
    /// 傍点に対応します。
    Boten(BotenKind),
    /// 字下げに対応します。
    Indent(usize),
    /// N字下げ、折り返してM字下げに対応します。
    Hanging((usize, usize)),
    /// 地付きに対応します。
    Grounded,
    /// 地からN地上げに対応します。
    LowFlying(usize),
    /// 大見出しに対応します。
    AHead,
    /// 中見出しに対応します。
    BHead,
    /// 小見出しに対応します。
    CHead,
    /// 縦中横に対応します。
    HinV,
    /// 「ママ」注記に対応します。
    Mama,
    /// N段階小さな文字に対応します。
    Smaller(usize),
    /// N段階大きな文字に対応します。
    Bigger(usize),
    /// ページ左右中央に対応します。
    VHCentre,
    /// 割り注に対応します。
    Warichu,
    ///　横組みに対応します。
    HorizontalLayout,
    /// 字詰めに対応します。
    Kerning(usize),
    /// 下付き小書き文字に対応します。
    Sub,
    /// 上付き小書き文字に対応します。
    Sup,
}

impl std::fmt::Display for Deco<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cow: Cow<str> = match self {
            Self::Bold => "太字".into(),
            Self::Italic => "斜体".into(),
            Self::Ruby(r) => format!("ルビ「{}」", r).into(),
            Self::Bosen(b) => b.to_string().into(),
            Self::Boten(b) => b.to_string().into(),
            Self::Indent(i) => format!("{}字下げ", i).into(),
            Self::Hanging(h) => format!("{}字下げ、折り返して{}字下げ", h.0, h.1).into(),
            Self::Grounded => "地付き".into(),
            Self::LowFlying(l) => format!("{}字寄せ", l).into(),
            Self::AHead => "大見出し".into(),
            Self::BHead => "中見出し".into(),
            Self::CHead => "小見出し".into(),
            Self::HinV => "縦中横".into(),
            Self::Mama => "ママ".into(),
            Self::Smaller(s) => format!("{}段階小さな文字", s).into(),
            Self::Bigger(s) => format!("{}段階大きな文字", s).into(),
            Self::VHCentre => "ページの左右中央".into(),
            Self::Warichu => "割注".into(),
            Self::HorizontalLayout => "横組み".into(),
            Self::Kerning(k) => format!("{}字詰め", k).into(),
            Self::Sub => "下付き小文字".into(),
            Self::Sup => "上付き小文字".into(),
        };
        write!(f, "[{}]", cow)
    }
}

/// 青空文庫書式形式で挿入される図表の意味を純化したものです。
///
/// aozora-rs-coreの時点ではIO操作を行わず、パス、キャプション、サイズ指定の情報のみを保持しています。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Figure<'s> {
    /// 指定された画像のパスを表します。
    ///
    /// 青空文庫で配布されているような.zip形式ならルートディレクトリが.zipのルートになります。
    pub path: &'s str,
    /// 画像に付加されているキャプションを表します。
    pub caption: &'s str,
    /// 画像のサイズを表します。単位はpxです。
    ///
    /// 実際の画像サイズではなく、実際に表示される段階でこのサイズで表示されることを要求するためのものです。
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

impl<'s> Into<AozoraTokenKind<'s>> for Figure<'s> {
    fn into(self) -> AozoraTokenKind<'s> {
        AozoraTokenKind::Note(Note::Single(Single::Figure(self)))
    }
}
