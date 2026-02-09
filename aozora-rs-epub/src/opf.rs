use aozora_rs_xhtml::XHTMLResult;
use chrono::prelude::*;
use uuid::Uuid;

use crate::zip::ImgExtension;

pub struct EpubMeta<'s> {
    pub title: &'s str,
    pub author: &'s str,
    pub language: &'s str,
    pub meta: DateTime<Local>,
    pub uuid: Uuid,
    pub styles: Vec<&'s str>,
    pub images: Vec<(&'s str, ImgExtension)>,
    pub xhtmls: XHTMLResult<'s>,
}

impl<'s> EpubMeta<'s> {
    fn uuid(author: &str, title: &str) -> Uuid {
        let namespace = Uuid::NAMESPACE_OID;
        let seed = format!("{}|{}", author, title);
        Uuid::new_v5(&namespace, seed.as_bytes())
    }

    pub fn new(
        title: &'s str,
        author: &'s str,
        language: &'s str,
        styles: Vec<&'s str>,
        images: Vec<(&'s str, ImgExtension)>,
        xhtmls: XHTMLResult<'s>,
    ) -> Self {
        Self {
            title,
            author,
            language,
            meta: Local::now(),
            uuid: Self::uuid(author, title),
            styles,
            images,
            xhtmls,
        }
    }

    pub fn into_opf(&self) -> String {
        let template = include_str!("../assets/template.opf");
        let styles = &self
            .styles
            .iter()
            .enumerate()
            .map(|(num, p)| {
                format!(
                    "       <item id=\"style{:>04}\" href=\"{}\" media-type=\"text/css\"/>",
                    num, p
                )
            })
            .collect::<Vec<String>>()
            .join("\n");
        let images = &self
            .images
            .iter()
            .enumerate()
            .map(|(num, (p, ext))| {
                format!(
                    "       <item id=\"image{:>04}\" href=\"{}\" media-type=\"image/{}\"/>",
                    num,
                    p,
                    ext.into_media_type()
                )
            })
            .collect::<Vec<String>>()
            .join("\n");
        let xhtmls = &self
            .xhtmls.xhtmls
            .iter()
            .enumerate()
            .map(|(num, p)| {
                format!(
                    "       <item id=\"xhtml{:>04}\" href=\"{}\" media-type=\"application/xhtml+xml\"/>",
                    num,
                    p
                )
            })
            .collect::<Vec<String>>()
            .join("\n");
        template
            .replace("［＃タイトル］", self.title)
            .replace("［＃言語コード］", self.language)
            .replace("［＃UUID］", &self.uuid.to_string())
            .replace(
                "［＃最終更新日］",
                &self.meta.format("YYYY-MM-DDThh:mm:ssZ").to_string(),
            )
            .replace("［＃著者］", self.author)
            .replace("［＃スタイル］", styles)
            .replace("［＃画像］", images)
            .replace("［＃XHTML］", xhtmls)
    }

    pub fn into_ncx(&self) -> String {
        let template = include_str!("../assets/toc.ncx");
        template
            .replace("［＃UUID］", &self.uuid.to_string())
            .replace("［＃タイトル］", self.title)
    }

    pub fn into_nav(&self) -> String {
        let navs = self
            .xhtmls
            .chapters
            .iter()
            .map(|c| format!("      <li><a href=\"{}\">{}</a></li>", c.get_nav(), c.name));
        include_str!("../assets/nav.xhtml")
            .replace("［＃章］", &navs.collect::<Vec<String>>().join("\n"))
    }
}
