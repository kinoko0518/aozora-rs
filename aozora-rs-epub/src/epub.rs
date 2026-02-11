//! # epub
//! epubを実際に生成する部分です。生成にあたって、以下のルールが適用されます。
//! - UUIDは著者名とタイトルを'|'で連結したものをSHA-1でハッシュ化したもの。
//! - xhtmlはitem/xhtmlの中に連番（sec0000.xhtml、sec0001.xhtml...）で配置される。idは拡張子を除いたファイル名と同じ。
//! - cssはitem/cssの中に連番（style0000.css、style0001.css）で配置される。idは拡張子を除いたファイル名と同じ。
//! - 画像はitem/imageの中に名前そのままで配置される。idは連番（image0000、image0001）

mod nav;
mod ncx;
mod opf;
mod xhtml;

use std::io::{Seek, Write};

use aozora_rs_core::{AZResult, AZResultC};
use chrono::Local;
use uuid::Uuid;
use zip::{ZipWriter, write::SimpleFileOptions};

use crate::{AozoraZip, ImgExtension};

pub struct EpubSetting<'s> {
    pub language: &'s str,
    pub is_rtl: bool,
}

pub struct EpubWriter<'s> {
    vzip: AozoraZip,
    setting: EpubSetting<'s>,
    lud: chrono::DateTime<Local>,
    styles: Vec<&'s str>,
}

impl EpubWriter<'_> {
    pub fn uuid(&self) -> Uuid {
        let namespace = Uuid::NAMESPACE_OID;
        let seed = format!("{}|{}", &self.vzip.nresult.author, &self.vzip.nresult.title);
        Uuid::new_v5(&namespace, seed.as_bytes())
    }

    pub fn xhtmls(&self) -> impl Iterator<Item = String> {
        self.vzip
            .nresult
            .xhtmls
            .xhtmls
            .iter()
            .enumerate()
            .map(|(num, _)| format!("xhtml/sec{:>04}.xhtml", num))
    }

    pub fn css(&self) -> impl Iterator<Item = String> {
        self.styles
            .iter()
            .enumerate()
            .map(|(num, _)| format!("style/style{:>04}.css", num))
    }

    pub fn images(&self) -> impl Iterator<Item = (String, ImgExtension)> {
        self.vzip
            .nresult
            .xhtmls
            .dependency
            .iter()
            .filter_map(|i| Some((i, ImgExtension::from_extension(i)?)))
            .map(|(i, e)| (format!("images/{i}"), e))
    }

    pub fn apply_css(&self, writer: &mut impl Write) -> Result<(), std::io::Error> {
        for css in self.css() {
            writeln!(
                writer,
                "\t\t<link rel=\"stylesheet\" type=\"text/css\" href=\"{}\" />",
                css
            )?;
        }
        Ok(())
    }
}

pub fn from_aozora_zip<T>(
    acc: impl Write + Seek,
    azz: AozoraZip,
    styles: Vec<&str>,
    setting: EpubSetting,
) -> Result<AZResult<()>, Box<dyn std::error::Error>> {
    let mut writer = ZipWriter::new(acc);
    let options = SimpleFileOptions::default();
    let epub_writer = EpubWriter {
        vzip: azz,
        setting,
        styles,
        lud: Local::now(),
    };

    writer.start_file("mimetype", options)?;
    writer.write_all(include_str!("../assets/mimetype").as_bytes())?;

    writer.start_file("META-INF/container.xml", options)?;
    writer.write_all(include_str!("../assets/container.xml").as_bytes())?;

    writer.start_file("item/standard.opf", options)?;
    epub_writer.write_opf(&mut writer)?;

    writer.start_file("item/toc.ncx", options)?;
    epub_writer.write_ncx(&mut writer)?;

    writer.start_file("item/nav.xhtml", options)?;
    epub_writer.write_nav(&mut writer)?;

    for (i, x) in epub_writer.vzip.nresult.xhtmls.xhtmls.iter().enumerate() {
        writer.start_file(format!("item/xhtml/sec{:>04}.xhtml", i), options)?;
        epub_writer.write_xhtml(&x, &mut writer)?;
    }

    for (i, css) in epub_writer.styles.iter().enumerate() {
        writer.start_file(format!("item/style/style{:>04}.css", i), options)?;
        writer.write_all(css.as_bytes())?;
    }

    let mut azresult = AZResultC::new();
    for d in epub_writer.vzip.nresult.xhtmls.dependency {
        if let Some(img) = epub_writer.vzip.images.get(&d) {
            writer.start_file(format!("item/image/{}", d), options)?;
            writer.write(&img.1)?;
        } else {
            azresult.push(
                miette::miette!("依存関係にあるファイルが見つかりませんでした：{}", d).into(),
            );
        }
    }

    Ok(azresult.finally(()))
}
