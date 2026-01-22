mod parser;
mod read_pdf;
mod satisfy;

use std::collections::HashMap;
use std::path::PathBuf;
use std::{env, fs};

use crate::parser::extract_gaiji_entries;
use crate::read_pdf::extract_from_pdf;
use crate::satisfy::gaiji_chuki;

type GaijiMap = HashMap<String, char>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let current: PathBuf = env::current_dir()?;

    let pdf_path = PathBuf::from(
        args.get(1)
            .map(|s| PathBuf::from(s))
            .unwrap_or(current.clone().join("assets")),
    );
    let output_path = args.get(2).map(|s| PathBuf::from(s)).unwrap_or(current);

    println!("gaiji_to_chuki.pdfからプレーンテキストを抽出中……");
    let plain = extract_from_pdf(&gaiji_chuki(&pdf_path)?)?;
    println!("gaiji_to_chuki.pdfから外字エントリを抽出中……");
    let map = extract_gaiji_entries(&mut plain.as_str()).unwrap();

    println!("外字エントリを保存中……");
    fs::write(
        &output_path.join("gaiji_to_char.map"),
        &rkyv::to_bytes::<_, 256>(&map)?,
    )?;

    println!("逆外字マップを作成中……");
    let mut reverse_map = HashMap::new();
    for (tag, kanji) in &map {
        reverse_map.insert(*kanji, tag.clone());
    }

    println!("逆外字マップを保存中……");
    fs::write(
        output_path.join("char_to_gaiji.map"),
        &rkyv::to_bytes::<_, 256>(&reverse_map)?,
    )?;

    println!("外字マップ抽出プロセスはすべて正常に終了しました。");
    Ok(())
}
