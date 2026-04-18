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

use aozora_rs_core::{AZResult, AZResultC, AozoraMeta};
use aozora_rs_xhtml::{Chapter, XHTMLResult};
use aozora_rs_zip::{Dependencies, ImgExtension};
use chrono::Local;
use uuid::Uuid;

use zip::{ZipWriter, result::ZipError, write::SimpleFileOptions};

/// Epubの生成に関する設定を保持する構造体です。
///
/// languageには言語コードを指定してください。is_rtlが真であれば縦書きのepubが生成されます。
pub struct EpubSetting<'s> {
    pub language: &'s str,
    pub is_rtl: bool,
    pub styles: Vec<&'s str>,
}

impl Default for EpubSetting<'_> {
    fn default() -> Self {
        Self {
            language: "ja",
            is_rtl: true,
            styles: Vec::new(),
        }
    }
}

/// 扉ページ生成に必要なデータ
pub struct TitlePageHyle<'a> {
    pub title: &'a str,
    pub author: &'a str,
}

/// 目次ページ生成に必要なデータ
pub struct TocPageHyle<'a> {
    pub chapters: &'a [Chapter],
}

/// EPUB生成時に注入可能なページ生成ロジック
pub struct PageInjectors {
    pub title_page: Option<Box<dyn Fn(&mut dyn Write, &TitlePageHyle) -> std::io::Result<()>>>,
    pub toc_page: Option<Box<dyn Fn(&mut dyn Write, &TocPageHyle) -> std::io::Result<()>>>,
}

impl Default for PageInjectors {
    fn default() -> Self {
        Self {
            title_page: None,
            toc_page: None,
        }
    }
}

/// epubの生成時に必要なデータをすべてまとめた構造体です。
///
/// epubを実際に生成する処理はこの構造体のメソッドとして実装されています。
pub(crate) struct EpubWriter<'s> {
    meta: &'s AozoraMeta<'s>,
    nresult: &'s XHTMLResult,
    image: &'s HashMap<String, (ImgExtension, Vec<u8>)>,
    setting: &'s EpubSetting<'s>,
    injectors: &'s PageInjectors,
    lud: chrono::DateTime<Local>,
}

impl EpubWriter<'_> {
    pub(crate) fn uuid(&self) -> Uuid {
        let namespace = Uuid::NAMESPACE_OID;
        let seed = format!("{}|{}", &self.meta.author, &self.meta.title);
        Uuid::new_v5(&namespace, seed.as_bytes())
    }

    pub(crate) fn has_title_page(&self) -> bool {
        self.injectors.title_page.is_some()
    }

    pub(crate) fn has_toc_page(&self) -> bool {
        self.injectors.toc_page.is_some()
    }

    pub(crate) fn xhtmls(&self) -> impl Iterator<Item = String> {
        self.nresult
            .xhtmls
            .iter()
            .enumerate()
            .map(|(num, _)| format!("xhtml/sec{:>04}.xhtml", num))
    }

    pub(crate) fn css(&self) -> impl Iterator<Item = String> {
        self.setting
            .styles
            .iter()
            .enumerate()
            .map(|(num, _)| format!("style/style{:>04}.css", num))
    }

    pub(crate) fn images(&self) -> impl Iterator<Item = (String, ImgExtension)> {
        self.nresult
            .dependency
            .iter()
            .filter_map(|i| Some((i, ImgExtension::from_extension(i)?)))
            .map(|(i, e)| (format!("images/{i}"), e))
    }

    pub(crate) fn apply_css(
        &self,
        writer: &mut impl Write,
        base_path: &str,
    ) -> Result<(), std::io::Error> {
        for (i, _) in self.setting.styles.iter().enumerate() {
            writeln!(
                writer,
                "\t<link rel=\"stylesheet\" type=\"text/css\" href=\"{}style{:>04}.css\" />",
                base_path, i
            )?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum EpubWarning {
    DependencieNotFound(String),
}

impl std::fmt::Display for EpubWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{}",
            match self {
                EpubWarning::DependencieNotFound(d) =>
                    format!("次のファイルがzip内に見つかりませんでした：{}", d),
            }
        )
    }
}

impl Default for EpubWarning {
    fn default() -> Self {
        Self::DependencieNotFound("".into())
    }
}

#[derive(Debug)]
pub enum AozoraEpubError {
    IoFailed(std::io::Error),
    ZipError(ZipError),
}

impl Into<AozoraEpubError> for std::io::Error {
    fn into(self) -> AozoraEpubError {
        AozoraEpubError::IoFailed(self)
    }
}

impl Into<AozoraEpubError> for ZipError {
    fn into(self) -> AozoraEpubError {
        AozoraEpubError::ZipError(self)
    }
}

impl std::fmt::Display for AozoraEpubError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::ZipError(z) => {
                    let err: String = match z {
                        ZipError::FileNotFound => "必要なファイルが見つかりませんでした".into(),
                        ZipError::InvalidArchive(_) => "無効なアーカイブです".into(),
                        ZipError::InvalidPassword => "Zipがパスワードで保護されています".into(),
                        ZipError::Io(i) => format!("IOエラーが発生しました：{}", i).into(),
                        ZipError::UnsupportedArchive(_) => {
                            "サポートされていないアーカイブ形式です".into()
                        }
                        _ => "".into(),
                    };
                    err
                }
                Self::IoFailed(i) => format!("IOエラーが発生しました：{}", i).into(),
            }
        )
    }
}

/// AozoraZipからEpubを生成します。
///
/// accには書き込み先を、settingにはEpubSettingを指定してください。
/// injectorsを指定すると、扉ページや目次ページを本文の前に挿入できます。
pub fn from_aozora_zip(
    acc: impl Write + Seek,
    dependencies: &Dependencies,
    xhtml: &XHTMLResult,
    setting: &EpubSetting,
    meta: &AozoraMeta,
    injectors: &PageInjectors,
) -> Result<AZResult<(), EpubWarning>, AozoraEpubError> {
    let mut writer = ZipWriter::new(acc);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    let epub_writer = EpubWriter {
        meta,
        nresult: xhtml,
        image: &dependencies.images,
        setting,
        injectors,
        lud: Local::now(),
    };
    let stored = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    writer
        .start_file("mimetype", stored)
        .map_err(|e| e.into())?;
    writer
        .write_all(b"application/epub+zip")
        .map_err(|e| e.into())?;

    writer
        .start_file("META-INF/container.xml", options)
        .map_err(|e| e.into())?;
    writer
        .write_all(include_str!("../assets/container.xml").as_bytes())
        .map_err(|e| e.into())?;

    writer
        .start_file("item/standard.opf", options)
        .map_err(|e| e.into())?;
    epub_writer.write_opf(&mut writer).map_err(|e| e.into())?;

    writer
        .start_file("item/toc.ncx", options)
        .map_err(|e| e.into())?;
    epub_writer.write_ncx(&mut writer).map_err(|e| e.into())?;

    writer
        .start_file("item/nav.xhtml", options)
        .map_err(|e| e.into())?;
    epub_writer.write_nav(&mut writer).map_err(|e| e.into())?;

    if let Some(ref title_writer) = injectors.title_page {
        writer
            .start_file("item/xhtml/title.xhtml", options)
            .map_err(|e| e.into())?;
        let hyle = TitlePageHyle {
            title: meta.title,
            author: meta.author,
        };
        epub_writer
            .write_injected_page(&mut writer, &hyle, title_writer.as_ref())
            .map_err(|e| e.into())?;
    }

    if let Some(ref toc_writer) = injectors.toc_page {
        writer
            .start_file("item/xhtml/toc.xhtml", options)
            .map_err(|e| e.into())?;
        let hyle = TocPageHyle {
            chapters: &xhtml.chapters,
        };
        epub_writer
            .write_injected_page(&mut writer, &hyle, toc_writer.as_ref())
            .map_err(|e| e.into())?;
    }

    for (i, x) in epub_writer.nresult.xhtmls.iter().enumerate() {
        writer
            .start_file(format!("item/xhtml/sec{:>04}.xhtml", i), options)
            .map_err(|e| e.into())?;
        epub_writer
            .write_xhtml(x, &mut writer)
            .map_err(|e| e.into())?;
    }

    for (i, css) in epub_writer.setting.styles.iter().enumerate() {
        writer
            .start_file(format!("item/style/style{:>04}.css", i), options)
            .map_err(|e| e.into())?;
        writer.write_all(css.as_bytes()).map_err(|e| e.into())?;
    }

    let mut azresult = AZResultC::default();
    for d in &epub_writer.nresult.dependency {
        if let Some(img) = epub_writer.image.get(d) {
            writer
                .start_file(format!("item/image/{}", d), options)
                .map_err(|e| e.into())?;
            writer.write(&img.1).map_err(|e| e.into())?;
        } else {
            azresult.acc_err(EpubWarning::DependencieNotFound(d.clone()));
        }
    }

    Ok(azresult.finally(()))
}
