use std::{
    collections::HashMap,
    fs,
    io::{Cursor, Read},
    path::Path,
};

use miette::Diagnostic;
use thiserror::Error;
use zip::result::ZipError;

pub enum ImgExtension {
    Png,
    Jpeg,
    Gif,
    Svg,
}

impl ImgExtension {
    pub fn into_media_type(&self) -> &str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpeg",
            Self::Gif => "gif",
            Self::Svg => "svg+xml",
        }
    }
}

pub struct AozoraZip {
    pub text: String,
    pub images: HashMap<String, (ImgExtension, Vec<u8>)>,
    pub css: HashMap<String, String>,
}

#[derive(Debug, Error, Diagnostic)]
pub enum AozoraZipError {
    #[error("ファイル操作中にエラーが発生しました")]
    #[diagnostic(
        code(aozora_rs_epub::io_error),
        help("ファイルパスや読み取り権限を確認してください。")
    )]
    Io(#[from] std::io::Error),

    #[error("複数のテキストファイルが見つかりました")]
    #[diagnostic(
        code(aozora_rs_epub::multi_textfile_found),
        help("対象になりうるテキストファイルは1つまでです。")
    )]
    MultiTextFound,

    #[error("テキストファイルが見つかりませんでした")]
    #[diagnostic(
        code(aozora_rs_epub::no_textfile_found),
        help("対象となるテキストファイルが必要です。")
    )]
    NoTextFound,

    #[error("Zipファイルの形式が不正です")]
    #[diagnostic(
        code(aozora_rs_epub::broken_zip_file),
        help("ファイルが破損していないかを確認してください。")
    )]
    BrokenZip(ZipError),
}

impl AozoraZip {
    pub fn read_from_zip_inner(zip: &[u8]) -> Result<Self, AozoraZipError> {
        let mut zip =
            zip::ZipArchive::new(Cursor::new(zip)).map_err(|e| AozoraZipError::BrokenZip(e))?;
        let mut images = HashMap::new();
        let mut css = HashMap::new();
        let mut txt = None;

        let zip_len = zip.len();
        for c in 0..zip_len {
            let c = zip.by_index(c).map_err(|e| AozoraZipError::BrokenZip(e));
            let mut c = c?;
            macro_rules! img_insert {
                ($ext:expr) => {{
                    let mut buff = Vec::new();
                    c.read_to_end(&mut buff)
                        .map_err(|e| AozoraZipError::Io(e))?;
                    images.insert(c.name().to_string(), ($ext, buff));
                }};
            }
            match c.name().rsplit_once(".").map(|(_, r)| r).unwrap_or("") {
                "jpg" | "jpeg" | "JPG" | "JPEG" => img_insert!(ImgExtension::Jpeg),
                "png" | "PNG" => img_insert!(ImgExtension::Png),
                "gif" | "GIF" => img_insert!(ImgExtension::Gif),
                "svg" | "SVG" => img_insert!(ImgExtension::Svg),
                "css" | "CSS" => {
                    let mut buff = String::new();
                    c.read_to_string(&mut buff)
                        .map_err(|e| AozoraZipError::Io(e))?;
                    css.insert(c.name().to_string(), buff);
                }
                "txt" => {
                    let mut buff = String::new();
                    c.read_to_string(&mut buff)
                        .map_err(|e| AozoraZipError::Io(e))?;
                    if txt.is_none() {
                        txt = Some(buff);
                    } else {
                        return Err(AozoraZipError::MultiTextFound);
                    }
                }
                _ => (),
            }
        }
        Ok(Self {
            text: if let Some(s) = txt {
                s
            } else {
                return Err(AozoraZipError::NoTextFound);
            },
            images,
            css,
        })
    }

    pub fn read_from_dir(path: &Path) -> Result<Self, miette::Error> {
        Self::read_from_dir_inner(path).map_err(|e| e.into())
    }

    fn read_from_dir_inner(path: &Path) -> Result<Self, AozoraZipError> {
        let mut text = None;
        let mut images: HashMap<String, (ImgExtension, Vec<u8>)> = HashMap::new();
        let mut custom_style: HashMap<String, String> = HashMap::new();

        for dir in fs::read_dir(path)? {
            let path = dir?.path();

            macro_rules! img_insert {
                ($ext:expr) => {
                    if let (Some(file_name), Some(path)) =
                        (path.file_name().and_then(|f| f.to_str()), path.to_str())
                    {
                        images.insert(file_name.to_string(), ($ext, fs::read(path)?));
                    }
                };
            }

            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                match ext {
                    "txt" => {
                        if text.is_none() {
                            text = Some(fs::read_to_string(&path)?);
                        } else {
                            return Err(AozoraZipError::MultiTextFound);
                        }
                    }
                    "jpg" | "JPG" | "jpeg" | "JPEG" => img_insert!(ImgExtension::Jpeg),
                    "png" | "PNG" => img_insert!(ImgExtension::Png),
                    "gif" | "GIF" => img_insert!(ImgExtension::Gif),
                    "svg" | "SVG" => img_insert!(ImgExtension::Svg),
                    "css" => {
                        if let (Some(file_name), Some(path)) =
                            (path.file_name().and_then(|f| f.to_str()), path.to_str())
                        {
                            custom_style.insert(file_name.to_string(), fs::read_to_string(path)?);
                        }
                    }
                    _ => (),
                }
            }
        }

        if let Some(text) = text {
            Ok(Self {
                text,
                images,
                css: custom_style,
            })
        } else {
            return Err(AozoraZipError::NoTextFound);
        }
    }
}
