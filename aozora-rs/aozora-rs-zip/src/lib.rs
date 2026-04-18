use std::{
    collections::HashMap,
    io::{Read, Seek},
    string::FromUtf8Error,
};

use winnow::error::ContextError;
use zip::result::ZipError;

#[derive(Debug, Clone, Copy)]
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
#[derive(Debug)]
pub enum DependenciesError {
    Io(std::io::Error),
    MultiTextFound,
    NoTextFound,
    BrokenZip(ZipError),
    EncodingError,
    BrokenText,
    TokenizeFailed(ContextError),
    BrokenMetaData,
    ImgReadFailed(std::io::Error),
}

impl std::fmt::Display for DependenciesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err: String = match self {
            Self::BrokenMetaData => "メタデータが破損しています".into(),
            Self::BrokenText => "入力されたテキストデータは破損しています".into(),
            Self::BrokenZip(z) => match z {
                ZipError::FileNotFound => "必要なファイルが見つかりませんでした".into(),
                ZipError::InvalidArchive(_) => "無効なアーカイブです".into(),
                ZipError::InvalidPassword => "Zipがパスワードで保護されています".into(),
                ZipError::Io(i) => format!("IOエラーが発生しました：{}", i),
                ZipError::UnsupportedArchive(_) => "サポートされていないアーカイブ形式です".into(),
                _ => "".into()
            },
            Self::EncodingError => "テキストデータの解釈に失敗しました。指定した文字コードが正しいことを確認してください".into(),
            Self::ImgReadFailed(i) => format!("画像の読み込みに失敗しました：{}", i).into(),
            Self::Io(i) => format!("I/Oに失敗しました：{}", i).into(),
            Self::MultiTextFound => "Zipの中に複数のテキストファイルが見つかりました".into(),
            Self::NoTextFound => "Zipの中にテキストファイルが見つかりませんでした".into(),
            Self::TokenizeFailed(_) => "テキストのトークン化に失敗しました".into()
        };
        write!(f, "{}", err)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Encoding {
    ShiftJIS,
    Utf8,
}

impl Encoding {
    pub fn bytes_to_string(&self, bytes: Vec<u8>) -> Result<String, FromUtf8Error> {
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
    pub images: Dependencies,
}

#[derive(Default, Clone)]
pub struct Dependencies {
    pub images: HashMap<String, (ImgExtension, Vec<u8>)>,
}

impl AozoraZip {
    pub fn read_from_zip<'s, T>(zip: T, encoding: &Encoding) -> Result<Self, DependenciesError>
    where
        T: Read + Seek,
    {
        let mut zip = zip::ZipArchive::new(zip).map_err(DependenciesError::BrokenZip)?;
        let mut images = HashMap::new();
        let mut txt = None;

        let zip_len = zip.len();
        for c in 0..zip_len {
            let c = zip.by_index(c).map_err(DependenciesError::BrokenZip);
            let mut c = c?;
            let extension = c.name().rsplit_once(".").map(|(_, r)| r).unwrap_or("");
            if extension == "txt" {
                let text = {
                    let mut buff: Vec<u8> = Vec::new();
                    c.read_to_end(&mut buff).map_err(DependenciesError::Io)?;
                    encoding
                        .bytes_to_string(buff)
                        .map_err(|_| DependenciesError::BrokenText)?
                };
                if txt.is_none() {
                    txt = Some(text);
                } else {
                    return Err(DependenciesError::MultiTextFound);
                }
            } else if let Some(ext) = ImgExtension::from_extension(extension) {
                let mut buff: Vec<u8> = Vec::new();
                c.read_to_end(&mut buff)
                    .map_err(DependenciesError::ImgReadFailed)?;
                images.insert(c.name().into(), (ext, buff));
            }
        }
        let nresult = if let Some(s) = txt {
            s
        } else {
            return Err(DependenciesError::NoTextFound);
        };
        Ok(Self {
            txt: nresult,
            images: Dependencies { images },
        })
    }
}
