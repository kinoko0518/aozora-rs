use aozora_rs_xhtml::{XHTMLResult, get_xhtml_filename};
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
    pub images: Vec<(String, ImgExtension)>,
    pub xhtmls: XHTMLResult,
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
        images: Vec<(String, ImgExtension)>,
        xhtmls: XHTMLResult,
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
            .map(|(num, _)| {
                format!(
                    "       <item id=\"sec{:>04}\" href=\"xhtml/xhtml{:>04}.xhtml\" media-type=\"application/xhtml+xml\"/>",
                    num,
                    num
                )
            })
            .collect::<Vec<String>>()
            .join("\n");
        let spine = &self
            .xhtmls
            .xhtmls
            .iter()
            .enumerate()
            .map(|(num, _)| format!("\t\t<itemref linear=\"yes\" idref=\"sec{:>04}\" />", num))
            .collect::<Vec<String>>()
            .join("\n");
        template
            .replace("［＃タイトル］", self.title)
            .replace("［＃言語コード］", self.language)
            .replace("［＃UUID］", &self.uuid.to_string())
            .replace(
                "［＃最終更新日］",
                &self.meta.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            )
            .replace("［＃著者］", self.author)
            .replace("［＃スタイル］", styles)
            .replace("［＃画像］", images)
            .replace("［＃XHTML］", xhtmls)
            .replace("［＃スパイン］", spine)
    }

    pub fn into_ncx(&self) -> String {
        let template = include_str!("../assets/toc.ncx");
        template
            .replace("［＃UUID］", &self.uuid.to_string())
            .replace("［＃タイトル］", self.title)
            .replace("［＃最初のXHTML］", &get_xhtml_filename(0))
    }

    pub fn into_nav(&self) -> String {
        let navs = self
            .xhtmls
            .chapters
            .iter()
            .map(|c| format!("      <li><a href=\"{}\">{}</a></li>", c.get_nav(), c.name));
        include_str!("../assets/nav.xhtml")
            .replace("［＃タイトル］", self.title)
            .replace("［＃CSS］", "")
            .replace("［＃最初のXHTML］", &get_xhtml_filename(0))
            .replace("［＃章］", &navs.collect::<Vec<String>>().join("\n"))
    }
}
