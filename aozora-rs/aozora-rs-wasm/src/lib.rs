use std::io::Cursor;

use aozora_rs::{AozoraDocument, AozoraWarning, AozoraZip, utf8tify_all_gaiji};
use wasm_bindgen::{JsError, prelude::wasm_bindgen};

#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct StandaloneXHTML {
    #[wasm_bindgen(getter_with_clone)]
    pub result: String,
    #[wasm_bindgen(getter_with_clone)]
    pub occured_error: String,
}

pub fn warnings_to_string(errors: Vec<AozoraWarning>, original: impl AsRef<str>) -> String {
    errors
        .iter()
        .map(|e| e.display(original.as_ref()))
        .collect::<Vec<String>>()
        .join("\n")
}

#[wasm_bindgen]
pub fn generate_standalone_xhtml(from: &str, delimiter: &str) -> Result<StandaloneXHTML, JsError> {
    let converted = utf8tify_all_gaiji(from);
    let doc = AozoraDocument::from_str(&converted, None)?;
    let (xhtml, errors) = doc.xhtml()?;
    Ok(StandaloneXHTML {
        result: xhtml.xhtmls.join(delimiter),
        occured_error: warnings_to_string(errors, converted),
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
    let converted = utf8tify_all_gaiji(from);
    let doc = AozoraDocument::from_str(&converted, None)?;
    let (xhtml, errors) = doc.xhtml()?;
    Ok(BookData {
        title: doc.meta.title.into(),
        author: doc.meta.author.into(),
        xhtmls: xhtml.xhtmls,
        errors: warnings_to_string(errors, converted),
    })
}

#[wasm_bindgen]
pub fn build_epub_bytes(
    from: &[u8],
    styles: Vec<String>,
    encoding: &str,
) -> Result<Vec<u8>, JsError> {
    let mut acc = Cursor::new(Vec::new());
    let styles: Vec<&str> = styles.iter().map(|s| s.as_str()).collect();
    AozoraDocument::from_zip(
        &AozoraZip::read_from_zip(
            Cursor::new(from),
            &match encoding {
                "utf8" | "utf-8" | "UTF8" | "UTF-8" => aozora_rs_zip::Encoding::Utf8,
                _ => aozora_rs_zip::Encoding::ShiftJIS,
            },
        )
        .map_err(|_| JsError::new("ZipからのAozoraDocumentの構築に失敗しました"))?,
    )?
    .epub(
        &mut acc,
        &aozora_rs::Style::default().extend_css(styles.into_iter()),
        &aozora_rs::PageInjectors::default(),
    )
    .map_err(|_| JsError::new("EPUB生成中にエラーが発生しました"))?;
    Ok(acc.into_inner())
}
