use std::io::Cursor;

use aozora_rs::{
    AozoraDocument, AozoraWarning, AozoraZip, Dependencies, Encoding, Style, WritingDirection,
    internal::AozoraMeta, utf8tify_all_gaiji,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsError, JsValue, prelude::wasm_bindgen};

#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen(typescript_custom_section)]
const TS_DEFINITIONS: &'static str = include_str!("definitions.d.ts");


#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpubOptions {
    pub encoding: Option<String>,
    pub is_vertical: Option<bool>,
    pub use_miyabi: Option<bool>,
    pub use_prelude: Option<bool>,
    pub consider_gaiji: Option<bool>,
}

impl Default for EpubOptions {
    fn default() -> Self {
        Self {
            encoding: None,
            is_vertical: Some(true),
            use_miyabi: Some(true),
            use_prelude: Some(true),
            consider_gaiji: Some(true),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HtmlOptions {
    pub encoding: Option<String>,
    pub is_vertical: Option<bool>,
    pub use_miyabi: Option<bool>,
    pub use_prelude: Option<bool>,
    pub consider_gaiji: Option<bool>,
}

impl Default for HtmlOptions {
    fn default() -> Self {
        Self {
            encoding: None,
            is_vertical: Some(true),
            use_miyabi: Some(true),
            use_prelude: Some(true),
            consider_gaiji: Some(true),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParseOptions {
    pub encoding: Option<String>,
    pub consider_gaiji: Option<bool>,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            encoding: None,
            consider_gaiji: Some(true),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedChapter {
    pub xhtml_id: usize,
    pub name: String,
    pub id: String,
    pub nav: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedBook {
    pub title: String,
    pub author: String,
    pub xhtmls: Vec<String>,
    pub chapters: Vec<ParsedChapter>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewResult {
    pub html: String,
    pub warnings: Vec<String>,
}

// --- 内部ヘルパー関数 ---

fn parse_options<T: for<'de> Deserialize<'de> + Default>(val: Option<JsValue>) -> T {
    match val {
        Some(v) if !v.is_null() && !v.is_undefined() => {
            serde_wasm_bindgen::from_value(v).unwrap_or_default()
        }
        _ => T::default(),
    }
}

fn decode_bytes_to_str(bytes: &[u8], encoding_opt: Option<&str>) -> Result<String, JsError> {
    match encoding_opt.map(|s| s.trim().to_lowercase()) {
        Some(ref enc) if enc == "shift_jis" || enc == "sjis" => {
            let (cow, _, _) = encoding_rs::SHIFT_JIS.decode(bytes);
            Ok(cow.into_owned())
        }
        Some(ref enc) if enc == "utf-8" || enc == "utf8" => {
            std::str::from_utf8(bytes)
                .map(|s| s.to_string())
                .map_err(|e| JsError::new(&format!("UTF-8のデコードに失敗しました: {e}")))
        }
        _ => {
            // 自動判定: UTF-8 として正当なら UTF-8、そうでなければ Shift_JIS にフォールバック
            if let Ok(s) = std::str::from_utf8(bytes) {
                Ok(s.to_string())
            } else {
                let (cow, _, _) = encoding_rs::SHIFT_JIS.decode(bytes);
                Ok(cow.into_owned())
            }
        }
    }
}

fn load_input_to_text_and_deps(
    input: &[u8],
    encoding_opt: Option<&str>,
) -> Result<(String, Option<Dependencies>), JsError> {
    // 先頭が PK\x03\x04 なら ZIP アーカイブと判定
    if input.starts_with(b"PK\x03\x04") {
        let azz = match encoding_opt.map(|s| s.trim().to_lowercase()) {
            Some(ref s) if s == "utf-8" || s == "utf8" => {
                AozoraZip::read_from_zip(Cursor::new(input), &Encoding::Utf8)
            }
            Some(ref s) if s == "shift_jis" || s == "sjis" => {
                AozoraZip::read_from_zip(Cursor::new(input), &Encoding::ShiftJIS)
            }
            _ => {
                // 自動判定: まず UTF-8 として展開を試み、失敗したら Shift_JIS でフォールバック
                match AozoraZip::read_from_zip(Cursor::new(input), &Encoding::Utf8) {
                    Ok(a) => Ok(a),
                    Err(_) => AozoraZip::read_from_zip(Cursor::new(input), &Encoding::ShiftJIS),
                }
            }
        }
        .map_err(|e| JsError::new(&format!("Zipファイルの展開に失敗しました: {e}")))?;
        Ok((azz.txt, Some(azz.images)))
    } else {
        let text = decode_bytes_to_str(input, encoding_opt)?;
        Ok((text, None))
    }
}

fn apply_gaiji_filter(text: &str, consider_gaiji: bool) -> (String, Vec<String>) {
    if consider_gaiji {
        let (converted, errs) = utf8tify_all_gaiji(text);
        let warnings = errs.into_iter().map(|s| format!("外字変換失敗: {s}")).collect();
        (converted.into_owned(), warnings)
    } else {
        (text.to_string(), Vec::new())
    }
}

fn build_style(is_vertical: bool, use_miyabi: bool, use_prelude: bool) -> Style<'static> {
    let mut style = Style::default();
    let direction = if is_vertical {
        WritingDirection::Vertical
    } else {
        WritingDirection::Horizontal
    };
    style.direction(direction).prelude(use_prelude);
    if use_miyabi {
        ayame::apply_miyabi(&mut style);
    }
    style
}

fn format_warnings(warnings: Vec<AozoraWarning>, original: &str) -> Vec<String> {
    warnings.iter().map(|w| w.display(original)).collect()
}

// --- 公開 WASM API ---

/// テキスト中の外字注記（`※［＃...］`）を Unicode (UTF-8) に一括変換します。
#[wasm_bindgen]
pub fn convert_gaiji(text: &str) -> String {
    let (converted, _) = utf8tify_all_gaiji(text);
    converted.into_owned()
}

/// エディタのリアルタイムタイピングプレビュー向けの最速変換関数です。
///
/// 構造体やメタデータの不要なオーバーヘッドを排除し、XHTML本文断片のみを高速に返します。
/// タイトル行などのヘッダが存在しない断片的なテキストでも安全にプレビューできます。
#[wasm_bindgen]
pub fn render_preview(text: &str) -> Result<String, JsError> {
    let (converted, _) = utf8tify_all_gaiji(text);
    // メタデータパースを試みる。失敗時は本文のみとしてパース
    let doc = match AozoraDocument::from_str(&converted, None) {
        Ok(d) => d,
        Err(_) => AozoraDocument::from_str_and_meta(
            AozoraMeta {
                title: "",
                author: "",
            },
            &converted,
            None,
        ),
    };
    let (xhtml, _) = doc.xhtml()?;
    Ok(xhtml.xhtmls.join("\n"))
}

/// プレビュー用 HTML とパース中に発生した警告一覧を返します。
#[wasm_bindgen]
pub fn render_preview_with_warnings(text: &str) -> Result<JsValue, JsError> {
    let (converted, mut gaiji_warnings) = apply_gaiji_filter(text, true);
    let doc = match AozoraDocument::from_str(&converted, None) {
        Ok(d) => d,
        Err(_) => AozoraDocument::from_str_and_meta(
            AozoraMeta {
                title: "",
                author: "",
            },
            &converted,
            None,
        ),
    };
    let (xhtml, warnings) = doc.xhtml()?;
    let mut all_warnings = format_warnings(warnings, &converted);
    all_warnings.append(&mut gaiji_warnings);

    let result = PreviewResult {
        html: xhtml.xhtmls.join("\n"),
        warnings: all_warnings,
    };
    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsError::new(&format!("シリアライズに失敗しました: {e}")))
}

/// テキスト文字列から書籍データ（タイトル、著者、章構造、XHTML本文、警告）をパースします。
#[wasm_bindgen]
pub fn parse_book(text: &str, options: Option<JsValue>) -> Result<JsValue, JsError> {
    let opts: ParseOptions = parse_options(options);
    let (converted, mut gaiji_warnings) =
        apply_gaiji_filter(text, opts.consider_gaiji.unwrap_or(true));

    let doc = AozoraDocument::from_str(&converted, None)?;
    let (xhtml, warnings) = doc.xhtml()?;
    let mut all_warnings = format_warnings(warnings, &converted);
    all_warnings.append(&mut gaiji_warnings);

    let chapters = xhtml
        .chapters
        .into_iter()
        .map(|c| ParsedChapter {
            id: c.get_id(),
            nav: c.get_nav(),
            xhtml_id: c.xhtml_id,
            name: c.name,
        })
        .collect();

    let book = ParsedBook {
        title: doc.meta.title.to_string(),
        author: doc.meta.author.to_string(),
        xhtmls: xhtml.xhtmls,
        chapters,
        warnings: all_warnings,
    };
    serde_wasm_bindgen::to_value(&book)
        .map_err(|e| JsError::new(&format!("シリアライズに失敗しました: {e}")))
}

/// ZIP または TXT バイト列から書籍データをパースします。
#[wasm_bindgen]
pub fn parse_book_from_bytes(input: &[u8], options: Option<JsValue>) -> Result<JsValue, JsError> {
    let opts: ParseOptions = parse_options(options);
    let (raw_text, deps) = load_input_to_text_and_deps(input, opts.encoding.as_deref())?;
    let (converted, mut gaiji_warnings) =
        apply_gaiji_filter(&raw_text, opts.consider_gaiji.unwrap_or(true));

    let doc = AozoraDocument::from_str(&converted, deps.as_ref())?;
    let (xhtml, warnings) = doc.xhtml()?;
    let mut all_warnings = format_warnings(warnings, &converted);
    all_warnings.append(&mut gaiji_warnings);

    let chapters = xhtml
        .chapters
        .into_iter()
        .map(|c| ParsedChapter {
            id: c.get_id(),
            nav: c.get_nav(),
            xhtml_id: c.xhtml_id,
            name: c.name,
        })
        .collect();

    let book = ParsedBook {
        title: doc.meta.title.to_string(),
        author: doc.meta.author.to_string(),
        xhtmls: xhtml.xhtmls,
        chapters,
        warnings: all_warnings,
    };
    serde_wasm_bindgen::to_value(&book)
        .map_err(|e| JsError::new(&format!("シリアライズに失敗しました: {e}")))
}

/// テキスト文字列からブラウザで直接閲覧可能な完全なスタンドアロン HTML を生成します。
///
/// スタイルシート（prelude / miyabi）およびスクロールスクリプトがインライン展開されます。
#[wasm_bindgen]
pub fn render_standalone_html(text: &str, options: Option<JsValue>) -> Result<String, JsError> {
    let opts: HtmlOptions = parse_options(options);
    let (converted, _) = apply_gaiji_filter(text, opts.consider_gaiji.unwrap_or(true));
    let doc = AozoraDocument::from_str(&converted, None)?;
    let style = build_style(
        opts.is_vertical.unwrap_or(true),
        opts.use_miyabi.unwrap_or(true),
        opts.use_prelude.unwrap_or(true),
    );

    let (html, _) = ayame::to_browser_xhtml(&doc, &style)?;
    Ok(html)
}

/// ZIP または TXT バイト列からブラウザで直接閲覧可能な完全なスタンドアロン HTML を生成します。
#[wasm_bindgen]
pub fn render_standalone_html_from_bytes(
    input: &[u8],
    options: Option<JsValue>,
) -> Result<String, JsError> {
    let opts: HtmlOptions = parse_options(options);
    let (raw_text, deps) = load_input_to_text_and_deps(input, opts.encoding.as_deref())?;
    let (converted, _) = apply_gaiji_filter(&raw_text, opts.consider_gaiji.unwrap_or(true));

    let doc = AozoraDocument::from_str(&converted, deps.as_ref())?;
    let style = build_style(
        opts.is_vertical.unwrap_or(true),
        opts.use_miyabi.unwrap_or(true),
        opts.use_prelude.unwrap_or(true),
    );

    let (html, _) = ayame::to_browser_xhtml(&doc, &style)?;
    Ok(html)
}

/// ZIP または TXT バイト列から EPUB バイナリを生成します。
///
/// 入力が ZIP ファイルかテキストファイルかは自動判別されます。
#[wasm_bindgen]
pub fn build_epub(input: &[u8], options: Option<JsValue>) -> Result<Vec<u8>, JsError> {
    let opts: EpubOptions = parse_options(options);
    let (raw_text, deps) = load_input_to_text_and_deps(input, opts.encoding.as_deref())?;
    let (converted, _) = apply_gaiji_filter(&raw_text, opts.consider_gaiji.unwrap_or(true));

    let style = build_style(
        opts.is_vertical.unwrap_or(true),
        opts.use_miyabi.unwrap_or(true),
        opts.use_prelude.unwrap_or(true),
    );

    let injectors = aozora_rs::PageInjectors {
        title_page: Some(ayame::title_page_writer()),
        toc_page: Some(ayame::toc_page_writer()),
    };

    let mut acc = Cursor::new(Vec::new());
    let doc = AozoraDocument::from_str(&converted, deps.as_ref())?;
    doc.epub(&mut acc, &style, &injectors)
        .map_err(|e| JsError::new(&format!("EPUB生成中にエラーが発生しました: {e}")))?;

    Ok(acc.into_inner())
}

/// テキスト文字列から直接 EPUB バイナリを生成します。
#[wasm_bindgen]
pub fn build_epub_from_str(text: &str, options: Option<JsValue>) -> Result<Vec<u8>, JsError> {
    let opts: EpubOptions = parse_options(options);
    let (converted, _) = apply_gaiji_filter(text, opts.consider_gaiji.unwrap_or(true));

    let style = build_style(
        opts.is_vertical.unwrap_or(true),
        opts.use_miyabi.unwrap_or(true),
        opts.use_prelude.unwrap_or(true),
    );

    let injectors = aozora_rs::PageInjectors {
        title_page: Some(ayame::title_page_writer()),
        toc_page: Some(ayame::toc_page_writer()),
    };

    let mut acc = Cursor::new(Vec::new());
    let doc = AozoraDocument::from_str(&converted, None)?;
    doc.epub(&mut acc, &style, &injectors)
        .map_err(|e| JsError::new(&format!("EPUB生成中にエラーが発生しました: {e}")))?;

    Ok(acc.into_inner())
}
