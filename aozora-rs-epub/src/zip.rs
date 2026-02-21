use std::{
    collections::HashMap,
    io::{Cursor, Read},
    string::FromUtf8Error,
};

use miette::Diagnostic;
use thiserror::Error;
use winnow::error::ContextError;
use zip::result::ZipError;

use crate::ImgExtension;

/// Zipファイルから読み込んだ、青空文庫書式の解析に必要なデータを保持する構造体です。
pub struct AozoraZip {
    pub txt: Vec<u8>,
    pub images: HashMap<String, (ImgExtension, Vec<u8>)>,
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

impl AozoraZip {
    pub fn read_from_zip<'s>(zip: &[u8]) -> Result<Self, AozoraZipError> {
        let mut zip =
            zip::ZipArchive::new(Cursor::new(zip)).map_err(|e| AozoraZipError::BrokenZip(e))?;
        let images = HashMap::new();
        let mut txt = None;

        let zip_len = zip.len();
        for c in 0..zip_len {
            let c = zip.by_index(c).map_err(|e| AozoraZipError::BrokenZip(e));
            let mut c = c?;
            match c.name().rsplit_once(".").map(|(_, r)| r).unwrap_or("") {
                "txt" => {
                    let text = {
                        let mut buff: Vec<u8> = Vec::new();
                        c.read_to_end(&mut buff)
                            .map_err(|e| AozoraZipError::Io(e))?;
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
