use std::io::Cursor;

use aozora_rs::{
    AozoraZip, EpubSetting, NovelResult, XHTMLResult, from_aozora_zip, from_sjis_aozora_zip,
    from_utf8_aozora_zip, parse_meta, retokenized_to_xhtml, str_to_retokenized,
};
use encoding_rs::SHIFT_JIS;
use miette::Diagnostic;
use serde::Serialize;
use std::collections::HashMap;
use thiserror::Error;

/// 入力テキストの文字エンコーディング
pub enum Encoding {
    Utf8,
    ShiftJis,
}

/// EPUB生成時の書字方向
pub enum WritingDirection {
    /// 縦書き（右→左）
    Vertical,
    /// 横書き
    Horizontal,
}

/// 組み込みCSSの名前を解決して内容を返す
///
/// 対応する名前: `"prelude"`, `"miyabi"`
pub fn resolve_builtin_css(name: &str) -> Option<&'static str> {
    match name {
        "prelude" => Some(include_str!("../assets/prelude.css")),
        "miyabi" => Some(include_str!("../assets/miyabi.css")),
        _ => None,
    }
}

/// WritingDirectionに対応するレイアウトCSSを返す
pub fn layout_css(direction: &WritingDirection) -> &'static str {
    match direction {
        WritingDirection::Vertical => include_str!("../assets/vertical.css"),
        WritingDirection::Horizontal => include_str!("../assets/horizontal.css"),
    }
}

/// 所有権付きの青空文庫メタデータ
#[derive(Debug, Clone, Serialize)]
pub struct OwnedAozoraMeta {
    pub title: String,
    pub author: String,
}

#[derive(Debug, Error, Diagnostic)]
pub enum AyameError {
    #[error("Shift-JISデコード中にエラーが発生しました")]
    #[diagnostic(help("ファイルのエンコーディングを確認してください。"))]
    ShiftJisDecodeError,

    #[error("UTF-8として読み取れませんでした")]
    #[diagnostic(help("Shift-JISファイルの場合はsjisオプションを指定してください。"))]
    Utf8DecodeError,

    #[error("メタデータの解析に失敗しました: {0}")]
    MetadataError(String),

    #[error("トークン化に失敗しました: {0}")]
    TokenizeError(String),

    #[error("Zipファイルの処理中にエラーが発生しました: {0}")]
    ZipError(String),

    #[error("EPUB変換に失敗しました: {0}")]
    EpubError(String),

    #[error("サポートされていないファイル形式です: .{0}")]
    UnsupportedFormat(String),
}

/// バイト列を指定エンコーディングでテキストにデコードする
pub fn decode_bytes(bytes: &[u8], encoding: &Encoding) -> Result<String, AyameError> {
    match encoding {
        Encoding::ShiftJis => {
            let (decoded, _, had_errors) = SHIFT_JIS.decode(bytes);
            if had_errors {
                return Err(AyameError::ShiftJisDecodeError);
            }
            Ok(decoded.replace("\r\n", "\n"))
        }
        Encoding::Utf8 => {
            String::from_utf8(bytes.to_vec()).map_err(|_| AyameError::Utf8DecodeError)
        }
    }
}

/// テキストからNovelResultを生成する
pub fn text_to_novel_result<'s>(text: &'s str) -> Result<NovelResult<'s>, AyameError> {
    let meta = parse_meta(text).map_err(|e| AyameError::MetadataError(e.to_string()))?;
    let az_result =
        str_to_retokenized(text).map_err(|e| AyameError::TokenizeError(e.to_string()))?;
    let (retokenized, errors) = az_result.into_tuple();
    Ok(retokenized_to_xhtml(retokenized, meta, errors))
}

/// 入力バイト列（txt or zip）からメタデータのみを取得する
pub fn scan_metadata(
    data: &[u8],
    is_zip: bool,
    encoding: &Encoding,
) -> Result<OwnedAozoraMeta, AyameError> {
    let text = if is_zip {
        let azz =
            AozoraZip::read_from_zip(data).map_err(|e| AyameError::ZipError(e.to_string()))?;
        decode_bytes(&azz.txt, encoding)?
    } else {
        decode_bytes(data, encoding)?
    };

    let meta = parse_meta(&text).map_err(|e| AyameError::MetadataError(e.to_string()))?;
    Ok(OwnedAozoraMeta {
        title: meta.title.to_string(),
        author: meta.author.to_string(),
    })
}

/// 入力バイト列からXHTMLを生成する
pub fn generate_xhtml(
    data: &[u8],
    is_zip: bool,
    encoding: &Encoding,
) -> Result<(XHTMLResult, Vec<miette::Report>), AyameError> {
    let text = if is_zip {
        let azz =
            AozoraZip::read_from_zip(data).map_err(|e| AyameError::ZipError(e.to_string()))?;
        decode_bytes(&azz.txt, encoding)?
    } else {
        decode_bytes(data, encoding)?
    };

    let result = text_to_novel_result(&text)?;
    Ok((result.xhtmls, result.errors))
}

/// 入力バイト列からEPUBバイト列を生成する
pub fn generate_epub(
    data: &[u8],
    is_zip: bool,
    encoding: &Encoding,
    styles: Vec<&str>,
    setting: EpubSetting,
) -> Result<Vec<u8>, AyameError> {
    let mut output = Cursor::new(Vec::new());

    if is_zip {
        match encoding {
            Encoding::ShiftJis => from_sjis_aozora_zip(&mut output, data, styles, setting),
            Encoding::Utf8 => from_utf8_aozora_zip(&mut output, data, styles, setting),
        }
        .map_err(|e| AyameError::EpubError(e.to_string()))?;
    } else {
        let text = decode_bytes(data, encoding)?;
        let novel_result = text_to_novel_result(&text)?;
        let azz = AozoraZip {
            txt: data.to_vec(),
            images: HashMap::new(),
        };
        from_aozora_zip(&mut output, azz, styles, setting, novel_result)
            .map_err(|e| AyameError::EpubError(e.to_string()))?;
    }

    Ok(output.into_inner())
}
