use std::{
    collections::HashMap,
    io::{Cursor, Read},
    string::FromUtf8Error,
};

use aozora_rs_xhtml::NovelResult;
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
    pub nresult: NovelResult,
    pub images: HashMap<String, (ImgExtension, Vec<u8>)>,
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

    #[error("エンコードエラーが発生しました")]
    #[diagnostic(
        code(aozora_rs_epub::encoding_error),
        help("Shift-JISファイルの場合は sjis オプションを有効にしてください。")
    )]
    EncodingError,

    #[error("txtファイルが破損しています")]
    #[diagnostic(
        code(aozora_rs_epub::encoding_error),
        help("Shift-JISファイルの場合は sjis オプションを有効にしてください。")
    )]
    BrokenText(FromUtf8Error),
}

impl AozoraZip {
    pub fn read_from_shift_jis_zip(zip: &[u8]) -> Result<Self, AozoraZipError> {
        Self::read_from_zip(
            zip,
            |data: Vec<u8>| -> Result<NovelResult, AozoraZipError> {
                let (encoded, _, _) = encoding_rs::SHIFT_JIS.decode(&data);
                Ok(aozora_rs_xhtml::convert_with_meta(&encoded))
            },
        )
    }

    pub fn read_from_utf8_zip(zip: &[u8]) -> Result<Self, AozoraZipError> {
        Self::read_from_zip(
            zip,
            |data: Vec<u8>| -> Result<NovelResult, AozoraZipError> {
                Ok(aozora_rs_xhtml::convert_with_meta(
                    &String::from_utf8(data).map_err(|e| AozoraZipError::BrokenText(e))?,
                ))
            },
        )
    }

    fn read_from_zip<'s>(
        zip: &[u8],
        txtf: impl Fn(Vec<u8>) -> Result<NovelResult, AozoraZipError>,
    ) -> Result<Self, AozoraZipError> {
        let mut zip =
            zip::ZipArchive::new(Cursor::new(zip)).map_err(|e| AozoraZipError::BrokenZip(e))?;
        let mut images = HashMap::new();
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
                "txt" => {
                    let text = {
                        let mut buff: Vec<u8> = Vec::new();
                        c.read(&mut buff).map_err(|e| AozoraZipError::Io(e))?;
                        buff
                    };
                    if txt.is_none() {
                        txt = Some(text);
                    } else {
                        return Err(AozoraZipError::MultiTextFound);
                    }
                }
                _ => (),
            }
        }
        let nresult = if let Some(s) = txt {
            txtf(s)?
        } else {
            return Err(AozoraZipError::NoTextFound);
        };
        Ok(Self { nresult, images })
    }
}
