use std::{
    borrow::Cow,
    io::{Seek, Write},
};

use aozora_rs::{
    AZResult, AZResultC, AozoraMeta, EpubSetting, NovelResult, into_xhtml, parse_meta,
    retokenized_to_novel_result, str_to_retokenized,
};
pub use aozora_rs_zip::Encoding;
use aozora_rs_zip::{AozoraZip, Dependencies};
use miette::miette;

#[derive(Default, Clone, Copy)]
pub enum WritingDirection {
    #[default]
    Vertical,
    Horizontal,
}

/// 内部的にCSSに変換される設定群の直積です。
///
/// コンテンツを正しく描画するための内部CSS、preludeをオフにする場合はuse_preludeをfalseに、
/// EPUBとして美しく表示するための内部CSS、miyabiをオフにする場合はuse_miyabiをfalseにしてください。
///
/// 縦書き / 横書きはdirectionで指定可能です。
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

/// ayameが解析可能なソースの直和です。
pub enum AozoraHyle {
    Txt((Vec<u8>, Encoding)),
    Zip((Vec<u8>, Encoding)),
}

impl AozoraHyle {
    /// 自身を`(String, Dependencies)`に変換します。
    pub fn encode(
        self,
        consider_gaiji: bool,
    ) -> Result<(String, Dependencies), Box<dyn std::error::Error>> {
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
        let text = if consider_gaiji {
            match aozora_rs::whole_gaiji_to_char(text.as_str()) {
                Cow::Owned(o) => o,
                Cow::Borrowed(_) => text,
            }
        } else {
            text
        };
        Ok((text, dependencies))
    }
}

#[derive(Clone)]
pub struct AbstractAozoraZip<'s> {
    pub meta: Option<AozoraMeta<'s>>,
    pub text: &'s str,
    pub dependencies: Dependencies,
}

fn str_to_novel_result<'s>(text: &'s str) -> Result<NovelResult<'s>, Box<dyn std::error::Error>> {
    let copied_str = &mut &*text;
    let meta = parse_meta(copied_str)?;
    let (retokenized, errors) = str_to_retokenized(copied_str)
        .map_err(|error| miette!(error))?
        .into_tuple();
    let novel_result = retokenized_to_novel_result(retokenized, meta, errors);
    Ok(novel_result)
}

impl<'s> AbstractAozoraZip<'s> {
    /// 与えられた&strをもとにメタデータを考慮して解析し、`AbstractAozoraZip`を構築します。
    /// メタデータの書き方は以下のURLの3-2. 基本となる書式を参照してください。
    ///
    /// https://www.aozora.gr.jp/aozora-manual/index-input.html
    pub fn from_str_with_meta(
        value: &'s str,
        dependencies: Dependencies,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let new = &mut &*value;
        Ok(Self {
            meta: Some(parse_meta(new)?),
            text: new,
            dependencies,
        })
    }

    /// 与えられた&strをもとに`AbstractAozoraZip`を構築します。
    pub fn from_str_no_meta(value: &'s str, dependencies: Dependencies) -> Self {
        Self {
            meta: None,
            text: value,
            dependencies,
        }
    }

    /// EPUBを`Seek` + `Write`を実装したaccに書き込みます。
    /// languageには言語コード（例: "ja"、"en"、"cn"）を入力してください。
    pub fn epub(
        self,
        acc: impl Seek + Write,
        potential: &PotentialCSS,
        language: &str,
    ) -> Result<AZResult<()>, Box<dyn std::error::Error>> {
        let novel_result = str_to_novel_result(self.text)?;
        aozora_rs_epub::from_aozora_zip(
            acc,
            self.dependencies,
            potential.to_epub_setting(language),
            novel_result,
        )
    }

    /// ブラウザでそのまま表示可能になるよう、CSSを埋め込んだり、
    /// <html>タグと<body>タグでラップしたりなどの処理を行って返します。
    pub fn browser_xhtml(
        self,
        potential: &PotentialCSS,
        mut css: Vec<&str>,
    ) -> Result<AZResult<String>, Box<dyn std::error::Error>> {
        let novel_result = str_to_novel_result(self.text)?;
        let (xhtmls, meta, errors) = (novel_result.xhtmls, novel_result.meta, novel_result.errors);
        potential.for_xhtml(&mut css);
        Ok(AZResultC::from(errors).finally(
            include_str!("../assets/base.xhtml")
                .replace("［＃タイトル］", meta.title)
                .replace("［＃スタイル］", &css.join("\n"))
                .replace("［＃本文］", &xhtmls.xhtmls.join("\n<hr>\n")),
        ))
    }

    /// 他のHTMLに埋め込む前提で生のXHTMLを生成します。
    pub fn embedding_xhtml(&self) -> Result<AZResult<String>, Box<dyn std::error::Error>> {
        let (retokenized, errors) = str_to_retokenized(self.text)
            .map_err(|error| miette!(error))?
            .into_tuple();
        let xhtml = into_xhtml(retokenized).xhtmls.join("\n");
        Ok(AZResultC::from(errors).finally(xhtml))
    }

    /// 文字列先頭のメタデータを読み、`AozoraMeta`を構築して返却します。文字列の消費は発生しません。
    pub fn scan_meta<'a>(&'a self) -> Result<AozoraMeta<'a>, miette::Report> {
        parse_meta(&mut &*self.text)
    }
}
