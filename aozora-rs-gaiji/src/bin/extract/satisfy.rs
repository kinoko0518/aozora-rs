use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

pub fn download(url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(url)
        .header("User-Agent", "gaiji-extractor")
        .send()?;

    if !response.status().is_success() {
        return Err(format!("Failed to download: {}", response.status()).into());
    }
    let content = response.bytes()?;
    Ok(content.into())
}

pub fn gaiji_chuki(path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let gaiji_chuki: PathBuf = path.join("gaiji_chuki.pdf");
    let url: &str = "https://www.aozora.gr.jp/gaiji_chuki/gaiji_chuki.pdf";
    if !std::fs::exists(&gaiji_chuki)? {
        println!(
            "{:?}にgaiji_chuki.pdfが見つかりませんでした。ダウンロードを試みます……",
            path
        );
        println!("ダウンロード中……");
        let downloaded = download(url)?;
        println!("gaiji_chuki.pdfを作成中……");
        let mut file = File::create(&gaiji_chuki)?;
        println!("ダウンロードしたデータを書き込み中……");
        file.write_all(&downloaded)?;
    }
    Ok(gaiji_chuki)
}

pub fn download_pdfium() -> Result<(), Box<dyn std::error::Error>> {
    let url =
        "https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-win-x64.tgz";

    println!("{}からPdfiumをダウンロード中……", url);
    let content = download(url)?;

    // 展開準備
    let reader = std::io::Cursor::new(content);
    let tar = flate2::read::GzDecoder::new(reader);
    let mut archive = tar::Archive::new(tar);

    let file_names = ["bin/pdfium.dll", "pdfium.dll"];
    let mut found = false;

    // 展開した中を探索
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.to_path_buf();
        let path_str = path.to_string_lossy();

        if file_names.iter().any(|name| path_str.ends_with(name)) {
            let mut out_file = std::fs::File::create("pdfium.dll")?;
            println!("{:?}からpdfium.dllを抽出中……", path);
            std::io::copy(&mut entry, &mut out_file)?;
            found = true;
            break;
        }
    }
    if !found {
        return Err("ダウンロードしたファイルにpdfium.dllは見つかりませんでした".into());
    }

    println!("Pdfiumのダウンロードと展開はすべて正常に終了しました");
    Ok(())
}
