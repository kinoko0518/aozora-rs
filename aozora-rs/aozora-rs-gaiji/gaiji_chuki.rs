use std::{
    collections::HashMap,
    fs::{self, File},
    hash::{DefaultHasher, Hash, Hasher},
    io::{Cursor, Write},
    path::Path,
};

use flate2::read::GzDecoder;
use gaiji_chuki_parser::{GaijiChuki, parse_tag};
use pdfium_render::prelude::{Pdfium, PdfiumError};
use rkyv::{rancor::Error, to_bytes};
use tar::Archive;
use winnow::{Parser, ascii::*, combinator::*, error::ContextError, token::take_until};

use crate::{ignore_rest_of_line, menkuten::MenkutenTable};

struct GaijiChukiLine<'s> {
    key: &'s str,
    value: GaijiChuki<'s>,
}

impl GaijiChukiLine<'_> {
    fn try_string(self, menkuten: &MenkutenTable) -> String {
        if let Some(uni) = self.value.unicode {
            return uni;
        } else if let Some(sjis) = self.value.sjis.and_then(|code| menkuten.get(&code)) {
            sjis.to_owned()
        } else {
            self.key.to_string()
        }
    }
}

pub async fn satisfy_pdfium(out_dir: &Path) -> Result<Pdfium, Box<dyn std::error::Error>> {
    let pdfium_url = if cfg!(target_os = "windows") {
        "https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-v8-win-x64.tgz"
    } else if cfg!(target_os = "macos") {
        "https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-v8-mac-univ.tgz"
    } else {
        "https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-v8-linux-x64.tgz"
    };
    let tgz = Cursor::new(reqwest::get(pdfium_url).await?.bytes().await?);
    let tar = GzDecoder::new(tgz);
    let mut archive = Archive::new(tar);
    let os_binname = if cfg!(target_os = "windows") {
        "pdfium.dll"
    } else if cfg!(target_os = "macos") {
        "libpdfium.dylib"
    } else {
        "libpdfium.so"
    };

    for entry in archive.entries()?.into_iter() {
        let query = if cfg!(target_os = "windows") {
            "bin/pdfium.dll"
        } else if cfg!(target_os = "macos") {
            "lib/libpdfium.dylib"
        } else {
            "lib/libpdfium.so"
        };
        let mut entry = entry?;
        if entry.path()?.to_str().unwrap_or("") == query {
            entry.unpack(out_dir.join(os_binname))?;
            break;
        }
    }

    let library_path = Pdfium::pdfium_platform_library_name_at_path(out_dir.to_str().unwrap());
    let bindings = Pdfium::bind_to_library(library_path)?;
    let pdfium = Pdfium::new(bindings);

    Ok(pdfium)
}

fn gaiji_chuki_line<'s>(input: &mut &'s str) -> Result<GaijiChukiLine<'s>, ContextError> {
    (take_until(0.., "※"), "※［＃", parse_tag, "］")
        .map(|(value, _, gcl, _): (&str, _, _, _)| GaijiChukiLine {
            key: value.trim(),
            value: gcl,
        })
        .parse_next(input)
}

fn collect_all_gaiji_chuki_line<'s>(
    input: &mut &'s str,
    menkuten: &MenkutenTable,
) -> HashMap<String, String> {
    input
        .lines()
        .into_iter()
        .filter_map(|mut line| {
            (
                space0,
                opt("★"),
                space0,
                opt((digit1, alt(("．　", "．", "． ")), space0)),
                gaiji_chuki_line,
                ignore_rest_of_line,
            )
                .map(|(_, _, _, _, gcl, _)| gcl)
                .parse_next(&mut line)
                .ok()
        })
        .fold(
            HashMap::new(),
            |mut acc: HashMap<String, String>, gcl: GaijiChukiLine| {
                let key = gcl.value.tag.replace(' ', "").replace('\u{3000}', "");
                let value = gcl.try_string(menkuten);
                acc.insert(key, value);
                acc
            },
        )
}

pub async fn get_latest_gaiji_chuki(
    pdfium: Pdfium,
    menkuten: &MenkutenTable,
    out_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://www.aozora.gr.jp/gaiji_chuki/gaiji_chuki.pdf";
    let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
            .build()?;
    let gaiji_chuki_pdf = client.get(url).send().await?.bytes().await?;
    let new_hash = {
        let mut hasher = DefaultHasher::new();
        gaiji_chuki_pdf.hash(&mut hasher);
        hasher.finish()
    };

    let pdf_path = out_dir.join("gaiji_chuki.pdf");

    let read = fs::read(&pdf_path).unwrap_or_default();
    let read_hash = {
        let mut hasher = DefaultHasher::new();
        read.hash(&mut hasher);
        hasher.finish()
    };

    if read_hash != new_hash {
        File::create(&pdf_path)?.write_all(&gaiji_chuki_pdf)?;
        let document = pdfium.load_pdf_from_file(pdf_path.to_str().unwrap(), None)?;
        let txt = {
            let tx: Result<String, PdfiumError> =
                document
                    .pages()
                    .iter()
                    .try_fold(String::new(), |mut acc: String, page| {
                        acc.extend(page.text()?.all().chars());
                        acc.push('\n');
                        Ok(acc)
                    });
            tx?
        };
        File::create(out_dir.join("read.txt"))?.write(txt.as_bytes())?;

        let gaiji_to_char = collect_all_gaiji_chuki_line(&mut txt.as_str(), menkuten);
        let char_to_gaiji: HashMap<String, String> = gaiji_to_char
            .clone()
            .into_iter()
            .map(|(k, v)| (v, k))
            .collect();

        File::create(out_dir.join("gaiji_to_char.map"))?
            .write_all(&to_bytes::<Error>(&gaiji_to_char)?)?;
        File::create(out_dir.join("char_to_gaiji.map"))?
            .write_all(&to_bytes::<Error>(&char_to_gaiji)?)?;
        File::create(out_dir.join("conversion_map.txt"))?.write_all(
            gaiji_to_char
                .iter()
                .map(|(key, value)| format!("K:\"{}\" -> V:\"{}\"", key, value))
                .collect::<Vec<String>>()
                .join("\n")
                .as_bytes(),
        )?;
    }
    Ok(())
}
