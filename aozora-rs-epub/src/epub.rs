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

use std::{
    collections::HashMap,
    io::{Seek, Write},
};

use aozora_rs_core::{AZResult, AZResultC, parse_meta, retokenize, scopenize, tokenize};
use aozora_rs_xhtml::NovelResult;
use chrono::Local;
use uuid::Uuid;
use winnow::LocatingSlice;
use zip::{ZipWriter, write::SimpleFileOptions};

use crate::{AozoraZip, AozoraZipError, ImgExtension};

/// Epubの生成に関する設定を保持する構造体です。
///
/// languageには言語コードを指定してください。is_rtlが真であれば縦書きのepubが生成されます。
pub struct EpubSetting<'s> {
    pub language: &'s str,
    pub is_rtl: bool,
}

/// epubの生成時に必要なデータをすべてまとめた構造体です。
///
/// epubを実際に生成する処理はこの構造体のメソッドとして実装されています。
pub struct EpubWriter<'s> {
    nresult: NovelResult<'s>,
    image: HashMap<String, (ImgExtension, Vec<u8>)>,
    setting: EpubSetting<'s>,
    lud: chrono::DateTime<Local>,
    styles: Vec<&'s str>,
}

impl EpubWriter<'_> {
    pub fn uuid(&self) -> Uuid {
        let namespace = Uuid::NAMESPACE_OID;
        let seed = format!("{}|{}", &self.nresult.meta.author, &self.nresult.meta.title);
        Uuid::new_v5(&namespace, seed.as_bytes())
    }

    pub fn xhtmls(&self) -> impl Iterator<Item = String> {
        self.nresult
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
        self.nresult
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

/// &strからNovelResultを生成します。
fn str_to_novel_result<'s>(str: &'s str) -> Result<NovelResult<'s>, AozoraZipError> {
    let meta = parse_meta(&mut &str).map_err(|e| AozoraZipError::BrokenMetaData(e))?;
    let tokenized =
        tokenize(&mut LocatingSlice::new(&str)).map_err(|e| AozoraZipError::TokenizeFailed(e))?;
    let ((scopenized, flat), error) = scopenize(tokenized, str).into_tuple();
    let retokenized = retokenize(flat, scopenized);
    Ok(aozora_rs_xhtml::retokenized_to_xhtml(
        retokenized,
        meta,
        error,
    ))
}

pub fn from_utf8_aozora_zip<T>(
    acc: &mut T,
    zip: &[u8],
    styles: Vec<&str>,
    setting: EpubSetting,
) -> Result<AZResult<()>, Box<dyn std::error::Error>>
where
    T: Write + Seek,
{
    let azz = AozoraZip::read_from_zip(zip)?;
    let txt = String::from_utf8(azz.txt.clone()).map_err(|e| AozoraZipError::BrokenText(e))?;
    let data = str_to_novel_result(&txt)?;
    from_aozora_zip(acc, azz, styles, setting, data)
}

pub fn from_sjis_aozora_zip<T>(
    acc: &mut T,
    zip: &[u8],
    styles: Vec<&str>,
    setting: EpubSetting,
) -> Result<AZResult<()>, Box<dyn std::error::Error>>
where
    T: Write + Seek,
{
    let azz = AozoraZip::read_from_zip(zip)?;
    let txt = {
        let (decoded, _, _) = encoding_rs::SHIFT_JIS.decode(&azz.txt);
        decoded.into_owned()
    };
    let data = str_to_novel_result(&txt)?;
    from_aozora_zip(acc, azz, styles, setting, data)
}

/// AozoraZipからEpubを生成します。
///
/// accには書き込み先を、azzには元となるAozoraZipを、stylesには使用するCSSを文字列として、settingにはEpubSettingを指定してください。
/// 最後にNovelResultを渡すことで、Epubを生成します。
pub fn from_aozora_zip<'s>(
    acc: impl Write + Seek,
    azz: AozoraZip,
    styles: Vec<&str>,
    setting: EpubSetting,
    novel_result: NovelResult<'s>,
) -> Result<AZResult<()>, Box<dyn std::error::Error>> {
    let mut writer = ZipWriter::new(acc);
    let options = SimpleFileOptions::default();
    let epub_writer = EpubWriter {
        nresult: novel_result,
        image: azz.images,
        setting,
        styles,
        lud: Local::now(),
    };

    let stored = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    writer.start_file("mimetype", stored)?;
    writer.write_all(b"application/epub+zip")?;

    writer.start_file("META-INF/container.xml", options)?;
    writer.write_all(include_str!("../assets/container.xml").as_bytes())?;

    writer.start_file("item/standard.opf", options)?;
    epub_writer.write_opf(&mut writer)?;

    writer.start_file("item/toc.ncx", options)?;
    epub_writer.write_ncx(&mut writer)?;

    writer.start_file("item/nav.xhtml", options)?;
    epub_writer.write_nav(&mut writer)?;

    for (i, x) in epub_writer.nresult.xhtmls.xhtmls.iter().enumerate() {
        writer.start_file(format!("item/xhtml/sec{:>04}.xhtml", i), options)?;
        epub_writer.write_xhtml(&x, &mut writer)?;
    }

    for (i, css) in epub_writer.styles.iter().enumerate() {
        writer.start_file(format!("item/style/style{:>04}.css", i), options)?;
        writer.write_all(css.as_bytes())?;
    }

    let mut azresult = AZResultC::new();
    for d in epub_writer.nresult.xhtmls.dependency {
        if let Some(img) = epub_writer.image.get(&d) {
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
