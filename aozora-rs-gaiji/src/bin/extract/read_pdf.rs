use std::error::Error;
use std::path::Path;

use pdfium_render::prelude::Pdfium;

use crate::satisfy::download_pdfium;

pub fn get_pdfium_binding() -> Result<Pdfium, String> {
    let bindings = Pdfium::bind_to_system_library().or_else(|_| {
        println!("Pdfiumが見つかりませんでした。ダウンロードを試みます……");
        download_pdfium()
            .map_err(|e| format!("Pdfiumのダウンロードに失敗しました。エラー: {}", e))?;
        Pdfium::bind_to_system_library().map_err(|e| format!("Pdfiumの取得に失敗しました。: {}", e))
    })?;
    Ok(Pdfium::new(bindings))
}

pub fn extract_from_pdf(pdf_path: &Path) -> Result<String, Box<dyn Error>> {
    let pdfium = get_pdfium_binding()?;
    let document = pdfium.load_pdf_from_file(pdf_path, None)?;

    let mut full_text = String::new();

    for page in document.pages().iter() {
        let text = page.text()?;
        full_text.push_str(&text.all());
        full_text.push('\n');
    }
    Ok(full_text)
}
