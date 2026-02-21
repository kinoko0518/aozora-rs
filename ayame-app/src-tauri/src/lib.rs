use std::fs::File;
use std::io::Read;
use std::path::Path;

use anyhow::{Context, Result};
use aozora_rs::EpubSetting;
use ayame_core::{
    generate_epub, layout_css, resolve_builtin_css, scan_metadata, Encoding, WritingDirection,
};
use serde::Serialize;

#[derive(Serialize)]
pub struct NovelMetadata {
    pub title: String,
    pub author: String,
}

#[tauri::command]
fn scan_file(path: String) -> Result<NovelMetadata, String> {
    scan_file_inner(&path).map_err(|e| e.to_string())
}

fn to_encoding(encoding: &str) -> Encoding {
    if encoding == "sjis" {
        Encoding::ShiftJis
    } else {
        Encoding::Utf8
    }
}

fn is_zip(path: &Path) -> bool {
    path.extension().map_or(false, |ext| ext == "zip")
}

fn scan_file_inner(path: &str) -> Result<NovelMetadata> {
    let path = Path::new(path);
    let mut file = File::open(path).context("ファイルのオープンに失敗しました")?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .context("ファイルの読み込みに失敗しました")?;

    let meta = scan_metadata(&buffer, is_zip(path), &Encoding::Utf8)
        .or_else(|_| scan_metadata(&buffer, is_zip(path), &Encoding::ShiftJis))
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(NovelMetadata {
        title: meta.title,
        author: meta.author,
    })
}

#[tauri::command]
fn convert_file(
    path: String,
    css: Vec<String>,
    vertical: bool,
    encoding: String,
) -> Result<Vec<u8>, String> {
    convert_file_inner(&path, css, vertical, &encoding).map_err(|e| e.to_string())
}

fn convert_file_inner(
    path: &str,
    css: Vec<String>,
    vertical: bool,
    encoding: &str,
) -> Result<Vec<u8>> {
    let path = Path::new(path);
    let mut file = File::open(path).context("ファイルのオープンに失敗しました")?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .context("ファイルの読み込みに失敗しました")?;

    let direction = if vertical {
        WritingDirection::Vertical
    } else {
        WritingDirection::Horizontal
    };

    let mut css_contents = vec![layout_css(&direction).to_string()];

    for c in css {
        if let Some(builtin) = resolve_builtin_css(&c) {
            css_contents.push(builtin.to_string());
        } else {
            let css_path = Path::new(&c);
            if css_path.exists() {
                match std::fs::read_to_string(css_path) {
                    Ok(s) => css_contents.push(s),
                    Err(e) => eprintln!("Failed to read CSS file {}: {}", c, e),
                }
            } else {
                eprintln!("CSS file not found: {}", c);
            }
        }
    }

    let css_refs: Vec<&str> = css_contents.iter().map(|s| s.as_str()).collect();
    let setting = EpubSetting {
        language: "ja",
        is_rtl: vertical,
    };

    generate_epub(
        &buffer,
        is_zip(path),
        &to_encoding(encoding),
        css_refs,
        setting,
    )
    .map_err(|e| anyhow::anyhow!("EPUB変換に失敗しました: {}", e))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![scan_file, convert_file])
        .run(tauri::generate_context!())
        .expect("Tauriアプリケーションの実行中にエラーが発生しました");
}
