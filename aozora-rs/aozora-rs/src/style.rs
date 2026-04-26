use aozora_rs_epub::EpubSetting;

/// 縦書き・横書きを指定するための列挙型です。
#[derive(Debug, Default, Clone, Copy)]
pub enum WritingDirection {
    #[default]
    /// 縦書きに対応します
    Vertical,
    /// 横書きに対応します
    Horizontal,
}

/// aozora-rsで汎用的に用いるスタイル構造体です。
///
/// ビルダーパターンを採用していますが、たいていの場合は[`Style::default`]で事足りると思います。
#[derive(Debug, Clone)]
pub struct Style<'s> {
    /// テキストの記述方向です。
    pub direction: WritingDirection,
    use_prelude: bool,
    css: Vec<&'s str>,
    language: &'s str,
}

impl Default for Style<'_> {
    fn default() -> Self {
        Self {
            direction: WritingDirection::Vertical,
            use_prelude: true,
            css: Vec::new(),
            language: "ja",
        }
    }
}

impl<'s> Style<'s> {
    /// 縦書き・横書きの方向を変更します。
    pub fn direction(&mut self, direction: WritingDirection) -> &mut Self {
        self.direction = direction;
        self
    }

    /// falseを与えるとaozora-rsで変換したHTMLを正常に表示させるためのCSS、preludeを無効化できます。
    pub fn prelude(&mut self, using: bool) -> &mut Self {
        self.use_prelude = using;
        self
    }

    /// 追加のカスタムCSSを追加します。
    pub fn add_css(&mut self, css: &'s str) -> &mut Self {
        self.css.push(css);
        self
    }

    /// イテレータから追加のカスタムCSSを追加します。
    pub fn extend_css(&mut self, css: impl Iterator<Item = &'s str>) -> &mut Self {
        self.css.extend(css);
        self
    }

    /// EPUBの内部的な言語コードを変更します。デフォルトは`"ja"`です。
    ///
    /// たとえば日本語なら`"ja"`、英語なら`"en"`、中国語なら`"cn"`となります。
    pub fn language(&mut self, language: &'s str) -> &mut Self {
        self.language = language;
        self
    }

    /// ここまでに蓄積してきたCSSに加え、[`Style`]の設定に基づき、
    /// 必要なCSSを追加して[`Vec<&str>`]として返却します。
    pub fn css(&self) -> Vec<&'s str> {
        let mut css = self.css.clone();
        if self.use_prelude {
            css.push(include_str!("../css/prelude.css"));
        }
        css.push(match self.direction {
            WritingDirection::Horizontal => include_str!("../css/horizontal.css"),
            WritingDirection::Vertical => include_str!("../css/vertical.css"),
        });
        css
    }

    /// [`Style`]から[`EpubSetting`]を生成します。
    pub fn epub_setting(&'s self) -> EpubSetting<'s> {
        EpubSetting {
            language: self.language,
            is_rtl: matches!(self.direction, WritingDirection::Vertical),
            styles: self.css(),
        }
    }
}
