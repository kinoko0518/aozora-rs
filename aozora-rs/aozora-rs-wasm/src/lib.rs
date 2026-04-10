use aozora_rs::{
    AozoraMeta, EpubSetting, from_aozora_zip, parse_meta, retokenized_to_novel_result,
    str_to_retokenized,
};
use aozora_rs_zip::{AozoraZip, Encoding as ZipEncoding};
use std::io::Cursor;
use wasm_bindgen::{JsError, JsValue, prelude::wasm_bindgen};

#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

fn reports_to_single_string(reports: Vec<miette::Report>) -> String {
    reports
        .iter()
        .map(|r| format!("{:?}", r))
        .collect::<Vec<_>>()
        .join("\n")
}

#[wasm_bindgen]
pub struct StandaloneXHTML {
    #[wasm_bindgen(getter_with_clone)]
    pub result: String,
    #[wasm_bindgen(getter_with_clone)]
    pub occured_error: String,
}

#[wasm_bindgen]
pub fn generate_standalone_xhtml(from: &str, delimiter: &str) -> Result<StandaloneXHTML, JsError> {
    let meta = AozoraMeta {
        title: "",
        author: "",
    };

    // パースエラー時はコンソールに出力して空の結果を返す
    let parsed = match str_to_retokenized(from) {
        Ok(p) => p,
        Err(e) => {
            web_sys::console::error_1(&JsValue::from_str(&e.to_string()));
            return Ok(StandaloneXHTML {
                result: String::new(),
                occured_error: e.to_string(),
            });
        }
    };

    let (retokenized, errors) = parsed.into_tuple();
    let xhtml = retokenized_to_novel_result(retokenized, meta, errors);

    let report = reports_to_single_string(xhtml.errors);

    // 蓄積されたエラーをコンソールに出力
    if !report.is_empty() {
        let err_msg = report.clone();
        web_sys::console::error_1(&JsValue::from_str(&err_msg));
    }

    Ok(StandaloneXHTML {
        result: xhtml.xhtmls.xhtmls.join(delimiter),
        occured_error: report,
    })
}

#[wasm_bindgen]
pub struct BookData {
    #[wasm_bindgen(getter_with_clone)]
    pub title: String,
    #[wasm_bindgen(getter_with_clone)]
    pub author: String,
    #[wasm_bindgen(getter_with_clone)]
    pub xhtmls: Vec<String>,
    #[wasm_bindgen(getter_with_clone)]
    pub errors: String,
}

#[wasm_bindgen]
pub fn parse_to_book_data(from: &str) -> Result<BookData, JsError> {
    let mut body = from;

    // メタデータ解析エラー時は空のメタデータをフォールバックとして続行
    let meta = match parse_meta(&mut body) {
        Ok(m) => m,
        Err(e) => {
            web_sys::console::error_1(&JsValue::from_str(&e.to_string()));
            AozoraMeta {
                title: "",
                author: "",
            }
        }
    };

    let parsed = match str_to_retokenized(body) {
        Ok(p) => p,
        Err(e) => {
            web_sys::console::error_1(&JsValue::from_str(&e.to_string()));
            return Ok(BookData {
                title: meta.title.to_string(),
                author: meta.author.to_string(),
                xhtmls: vec![],
                errors: e.to_string(),
            });
        }
    };

    let (retokenized, errors) = parsed.into_tuple();
    let result = retokenized_to_novel_result(retokenized, meta, errors);

    let report = reports_to_single_string(result.errors);

    // 蓄積されたエラーをコンソールに出力
    if !report.is_empty() {
        let err_msg = report.clone();
        web_sys::console::error_1(&JsValue::from_str(&err_msg));
    }

    Ok(BookData {
        title: result.meta.title.to_string(),
        author: result.meta.author.to_string(),
        xhtmls: result.xhtmls.xhtmls,
        errors: report,
    })
}

#[wasm_bindgen]
pub fn build_epub_bytes(
    from: &[u8],
    styles: Vec<String>,
    encoding: &str,
) -> Result<Vec<u8>, JsError> {
    let mut acc = Cursor::new(Vec::new());

    let enc = match encoding {
        "utf8" => ZipEncoding::Utf8,
        _ => ZipEncoding::ShiftJIS,
    };

    // ZIP読み込みの致命的エラー
    let azz = match AozoraZip::read_from_zip(from, &enc) {
        Ok(a) => a,
        Err(e) => {
            web_sys::console::error_1(&JsValue::from_str(&e.to_string()));
            return Ok(Vec::new()); // エラー時は空のバイト列でOkを返す
        }
    };

    let (body_string, dependencies) = azz.into_dependencies();
    let mut body_slice = body_string.as_str();

    let meta = match parse_meta(&mut body_slice) {
        Ok(m) => m,
        Err(e) => {
            web_sys::console::error_1(&JsValue::from_str(&e.to_string()));
            AozoraMeta {
                title: "",
                author: "",
            }
        }
    };

    let parsed = match str_to_retokenized(body_slice) {
        Ok(p) => p,
        Err(e) => {
            web_sys::console::error_1(&JsValue::from_str(&e.to_string()));
            return Ok(Vec::new());
        }
    };

    let (retokenized, errors) = parsed.into_tuple();
    let novel_result = retokenized_to_novel_result(retokenized, meta, errors);

    let setting = EpubSetting {
        language: "ja",
        is_rtl: true,
        styles: styles.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
    };

    if let Err(e) = from_aozora_zip(&mut acc, dependencies, setting, novel_result) {
        web_sys::console::error_1(&JsValue::from_str(&e.to_string()));
        return Ok(Vec::new());
    }

    Ok(acc.into_inner())
}
