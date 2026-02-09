use aozora_rs_epub::AozoraZip;
use std::{fs::File, io::Cursor, path::Path};
use wasm_bindgen::{JsError, prelude::wasm_bindgen};

fn into_js_error<E: std::fmt::Display>(err: E) -> JsError {
    JsError::new(&format!("{}", err))
}

fn reports_to_single_string(reports: Vec<miette::Report>) -> String {
    let msg = reports
        .iter()
        .map(|r| format!("{:?}", r))
        .collect::<Vec<_>>()
        .join("\n");
    msg
}

#[wasm_bindgen]
pub struct StandaloneXHTML {
    #[wasm_bindgen(getter_with_clone)]
    pub result: String,
    #[wasm_bindgen(getter_with_clone)]
    pub occured_error: String,
}

/// 青空文庫書式の冒頭、終端の特別な表記、たとえば一行目はタイトル、
/// 二行目は著者といったルールを考慮せず、全文を純粋な青空文庫書式として解析、
/// 単一の埋め込み用XHTMLとして1つのStringにまとめます。
///
/// ご自身のサイト自体を青空文庫書式で記述する用途に便利です。
#[wasm_bindgen]
pub fn generate_standalone_xhtml(from: &str, delimiter: &str) -> StandaloneXHTML {
    let result = aozora_rs_xhtml::convert_with_no_meta(from);
    StandaloneXHTML {
        result: result.xhtmls.xhtmls.join(delimiter),
        occured_error: reports_to_single_string(result.errors),
    }
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

/// 青空文庫書式の冒頭、終端の特別な表記、たとえば一行目はタイトル、
/// 二行目は著者といったルールを考慮し、メタデータとXHTMLのベクタの直積を返します。
///
/// 青空文庫書式で書かれた作品を解析し、独自の方法で表示したい場合に便利です。
/// 注意点として、この方式では画像にはリンク切れが発生します。
///
/// 現在、XHTMLの画像タグ自体に画像を埋め込む方法を検討しています。詳細は以下のissueを参照してください。
///
/// https://github.com/kinoko0518/aozora-rs/issues/7
#[wasm_bindgen]
pub fn parse_to_book_data(from: &str) -> BookData {
    let result = aozora_rs_xhtml::convert_with_meta(from);
    BookData {
        title: result.title.to_string(),
        author: result.author.to_string(),
        xhtmls: result.xhtmls.xhtmls,
        errors: reports_to_single_string(result.errors),
    }
}

/// zipであることを期待するバイト列を受けとり、構築したepubのバイト列をオンメモリで構築して返します。
#[wasm_bindgen]
pub fn build_epub_bytes(from: &[u8]) -> Result<Vec<u8>, JsError> {
    let azz = AozoraZip::read_from_zip_inner(from).map_err(into_js_error)?;
    let mut acc = Cursor::new(Vec::new());
    aozora_rs_epub::from_aozora_zip::<Cursor<Vec<u8>>>(&mut acc, azz, Vec::new())
        .map_err(into_js_error)?;
    Ok(acc.into_inner())
}

/// zipであることを期待するバイト列を受けとり、指定されたパスに構築したepubの直接書き込みを行います。
///
/// WASMでは動作しないため、wasm_bindgenタグはつきません。
pub fn save_epub_to_file(from: &[u8], to: &Path) -> Result<(), JsError> {
    let azz = AozoraZip::read_from_zip_inner(from).map_err(into_js_error)?;
    let mut acc = File::create(to).map_err(into_js_error)?;
    aozora_rs_epub::from_aozora_zip::<File>(&mut acc, azz, Vec::new()).map_err(into_js_error)?;
    Ok(())
}
