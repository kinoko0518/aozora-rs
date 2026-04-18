use aozora_rs_epub::EpubSetting;

#[derive(Debug, Default, Clone, Copy)]
pub enum WritingDirection {
    #[default]
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone)]
pub struct Style<'s> {
    direction: WritingDirection,
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
    pub fn direction(&mut self, direction: WritingDirection) -> &mut Self {
        self.direction = direction;
        self
    }

    pub fn prelude(&mut self, using: bool) -> &mut Self {
        self.use_prelude = using;
        self
    }

    pub fn add_css(&mut self, css: &'s str) -> &mut Self {
        self.css.push(css);
        self
    }

    pub fn extend_css(&mut self, css: impl Iterator<Item = &'s str>) -> &mut Self {
        self.css.extend(css);
        self
    }

    pub fn language(&mut self, language: &'s str) -> &mut Self {
        self.language = language;
        self
    }

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

    pub fn into_epub_setting(&'s self) -> EpubSetting<'s> {
        EpubSetting {
            language: self.language,
            is_rtl: matches!(self.direction, WritingDirection::Vertical),
            styles: self.css(),
        }
    }
}
