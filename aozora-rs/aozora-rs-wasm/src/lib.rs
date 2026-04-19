use std::io::Cursor;

use aozora_rs::{
    AozoraDocument, AozoraWarning, AozoraZip, WritingDirection, internal::AozoraMeta,
    utf8tify_all_gaiji,
};
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
pub fn generate_embedding_xhtml(from: &str, delimiter: &str) -> Result<StandaloneXHTML, JsError> {
    let converted = utf8tify_all_gaiji(from);
    let doc = AozoraDocument::from_str_and_meta(
        AozoraMeta {
            title: "MOCK_TITLE",
            author: "MOCK_AUTHOR",
        },
        &converted,
        None,
    );
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
    encoding: &str,
    is_vertical: bool,
    use_miyabi: bool,
    use_prelude: bool,
    consider_gaiji: bool,
) -> Result<Vec<u8>, JsError> {
    let mut acc = Cursor::new(Vec::new());
    let enc = match encoding {
        "utf8" | "utf-8" | "UTF8" | "UTF-8" => aozora_rs_zip::Encoding::Utf8,
        _ => aozora_rs_zip::Encoding::ShiftJIS,
    };

    let azz = AozoraZip::read_from_zip(Cursor::new(from), &enc)
        .map_err(|_| JsError::new("ZipからのAozoraDocumentの構築に失敗しました"))?;

    let txt = if consider_gaiji {
        utf8tify_all_gaiji(&azz.txt).into_owned()
    } else {
        azz.txt.clone()
    };

    let direction = if is_vertical {
        WritingDirection::Vertical
    } else {
        WritingDirection::Horizontal
    };

    let mut style = aozora_rs::Style::default();
    style.direction(direction).prelude(use_prelude);
    if use_miyabi {
        ayame::apply_miyabi(&mut style);
    }

    let injectors = aozora_rs::PageInjectors {
        title_page: Some(ayame::title_page_writer()),
        toc_page: Some(ayame::toc_page_writer()),
    };

    AozoraDocument::from_str(&txt, Some(&azz.images))?
        .epub(&mut acc, &style, &injectors)
        .map_err(|e| JsError::new(&format!("EPUB生成中にエラーが発生しました: {}", e)))?;

    Ok(acc.into_inner())
}
