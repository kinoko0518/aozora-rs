use std::{
    collections::HashMap,
    io::{Cursor, Read},
    string::FromUtf8Error,
};

use miette::Diagnostic;
use thiserror::Error;
use winnow::error::ContextError;
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

    pub fn from_extension(from: &str) -> Option<Self> {
        match from {
            "jpg" | "jpeg" | "JPG" | "JPEG" => Some(Self::Jpeg),
            "png" | "PNG" => Some(Self::Png),
            "gif" | "GIF" => Some(Self::Gif),
            "svg" | "SVG" => Some(Self::Svg),
            _ => None,
        }
    }
}

/// AozoraZipからepubやXHTMLを生成するときに発生しうるエラーを列挙したエラー型です。
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

    #[error("トークン化に失敗しました")]
    #[diagnostic(
        code(aozora_rs_epub::encoding_error),
        help(
            "原理的に失敗しないエラーです。お手数ですが、公開できるものであれば入力したデータとともに開発者へご連絡ください。"
        )
    )]
    TokenizeFailed(ContextError),

    #[error("メタデータの解析に失敗しました")]
    #[diagnostic(
        code(aozora_rs_epub::encoding_error),
        help("入力が青空文庫書式に従っていることを確認してください。")
    )]
    BrokenMetaData(miette::Report),
}

pub enum Encoding {
    ShiftJIS,
    Utf8,
}

impl Encoding {
    fn bytes_to_string(&self, bytes: Vec<u8>) -> Result<String, FromUtf8Error> {
        match self {
            Self::ShiftJIS => {
                let (cow, _, _) = encoding_rs::SHIFT_JIS.decode(&bytes);
                Ok(cow.to_string())
            }
            Self::Utf8 => String::from_utf8(bytes),
        }
    }
}

/// Zipファイルから読み込んだ、青空文庫書式の解析に必要なデータを保持する構造体です。
pub struct AozoraZip {
    pub txt: String,
    pub images: HashMap<String, (ImgExtension, Vec<u8>)>,
}

#[derive(Default)]
pub struct Dependencies {
    pub images: HashMap<String, (ImgExtension, Vec<u8>)>,
}


impl AozoraZip {
    pub fn into_dependencies(self) -> (String, Dependencies) {
        (
            self.txt,
            Dependencies {
                images: self.images,
            },
        )
    }

    pub fn read_from_zip<'s>(zip: &[u8], encoding: &Encoding) -> Result<Self, AozoraZipError> {
        let mut zip = zip::ZipArchive::new(Cursor::new(zip)).map_err(AozoraZipError::BrokenZip)?;
        let images = HashMap::new();
        let mut txt = None;

        let zip_len = zip.len();
        for c in 0..zip_len {
            let c = zip.by_index(c).map_err(AozoraZipError::BrokenZip);
            let mut c = c?;
            if c.name().rsplit_once(".").map(|(_, r)| r).unwrap_or("") == "txt" {
                let text = {
                    let mut buff: Vec<u8> = Vec::new();
                    c.read_to_end(&mut buff).map_err(AozoraZipError::Io)?;
                    encoding
                        .bytes_to_string(buff)
                        .map_err(AozoraZipError::BrokenText)?
                };
                if txt.is_none() {
                    txt = Some(text);
                } else {
                    return Err(AozoraZipError::MultiTextFound);
                }
            }
        }
        let nresult = if let Some(s) = txt {
            s
        } else {
            return Err(AozoraZipError::NoTextFound);
        };
        Ok(Self {
            txt: nresult,
            images,
        })
    }
}
