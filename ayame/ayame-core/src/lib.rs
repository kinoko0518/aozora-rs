use std::io::{Seek, Write};

use aozora_rs::{
    AZResult, AZResultC, AozoraMeta, EpubSetting, NovelResult, parse_meta, retokenized_to_xhtml,
    str_to_retokenized,
};
pub use aozora_rs_zip::Encoding;
use aozora_rs_zip::{AozoraZip, Dependencies};

#[derive(Default, Clone, Copy)]
pub enum WritingDirection {
    #[default]
    Vertical,
    Horizontal,
}

pub struct PotentialCSS {
    pub use_prelude: bool,
    pub use_miyabi: bool,
    pub direction: WritingDirection,
}

impl PotentialCSS {
    fn miyabi_not_specified_yet<'a>(&self, css: &mut Vec<&'a str>, miyabi: &'a str) {
        if self.use_prelude {
            css.push(include_str!("../assets/prelude.css"));
        }
        if self.use_miyabi {
            css.push(miyabi);
        }
        css.push(match self.direction {
            WritingDirection::Vertical => include_str!("../assets/vertical.css"),
            WritingDirection::Horizontal => include_str!("../assets/horizontal.css"),
        });
    }
    fn for_xhtml(&self, css: &mut Vec<&str>) {
        self.miyabi_not_specified_yet(css, include_str!("../assets/miyabix.css"));
    }
    fn for_epub(&self, css: &mut Vec<&str>) {
        self.miyabi_not_specified_yet(css, include_str!("../assets/miyabi.css"));
    }
    pub fn to_epub_setting<'a>(&'a self, language: &'a str) -> EpubSetting<'a> {
        let mut css = Vec::new();
        self.for_epub(&mut css);
        EpubSetting {
            language,
            is_rtl: match self.direction {
                WritingDirection::Vertical => true,
                WritingDirection::Horizontal => false,
            },
            styles: css,
        }
    }
}

/// テキストからNovelResultを生成する
pub fn text_to_novel_result<'s>(
    text: &'s str,
) -> Result<NovelResult<'s>, Box<dyn std::error::Error>> {
    let mut body = text;
    let meta = parse_meta(&mut body)?;
    let az_result = str_to_retokenized(body).map_err(|_| "".to_string())?;
    let (retokenized, errors) = az_result.into_tuple();
    Ok(retokenized_to_xhtml(retokenized, meta, errors))
}

pub enum AozoraHyle {
    Txt((Vec<u8>, Encoding)),
    Zip((Vec<u8>, Encoding)),
}

#[derive(Clone)]
pub struct AbstractAozoraZip {
    pub text: String,
    pub dependencies: Dependencies,
}

impl TryInto<AbstractAozoraZip> for AozoraHyle {
    type Error = Box<dyn std::error::Error>;

    fn try_into(self) -> Result<AbstractAozoraZip, Self::Error> {
        let (text, dependencies): (String, Dependencies) = match self {
            Self::Txt((data, encoding)) => {
                let txt = encoding.bytes_to_string(data)?;
                (txt, Dependencies::default())
            }
            Self::Zip((zip, encoding)) => {
                let azz = AozoraZip::read_from_zip(&zip, &encoding)?;
                (azz.txt, Dependencies { images: azz.images })
            }
        };
        Ok(AbstractAozoraZip { text, dependencies })
    }
}

impl AbstractAozoraZip {
    pub fn generate_epub(
        self,
        acc: impl Seek + Write,
        potential: PotentialCSS,
        language: &str,
    ) -> Result<AZResult<()>, Box<dyn std::error::Error>> {
        let novel_result = text_to_novel_result(&self.text)?;
        aozora_rs_epub::from_aozora_zip(
            acc,
            self.dependencies,
            potential.to_epub_setting(language),
            novel_result,
        )
    }

    pub fn generate_browser_xhtml(
        self,
        potential: PotentialCSS,
        mut css: Vec<&str>,
    ) -> Result<AZResult<String>, Box<dyn std::error::Error>> {
        let novel_result = text_to_novel_result(&self.text)?;
        let (xhtmls, meta, errors) = (novel_result.xhtmls, novel_result.meta, novel_result.errors);
        potential.for_xhtml(&mut css);
        Ok(AZResultC::from(errors).finally(
            include_str!("../assets/base.xhtml")
                .replace("［＃タイトル］", meta.title)
                .replace("［＃スタイル］", &css.join("\n"))
                .replace("［＃本文］", &xhtmls.xhtmls.join("\n<hr>\n")),
        ))
    }

    pub fn scan_meta<'a>(&'a self) -> Result<AozoraMeta<'a>, miette::Report> {
        parse_meta(&mut self.text.as_str())
    }
}
