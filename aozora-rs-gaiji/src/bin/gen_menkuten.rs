// JIS X 0213 マッピングテーブル生成ツール
//
// このツールはJIS X 0213の公式マッピングテーブルを解析し、
// 面区点→Unicode変換テーブルをrkyv形式で出力します。
//
// 使用方法:
//   cargo run -p aozora-rs-gaiji --bin gen_menkuten [input_path] [output_path]

use std::collections::HashMap;
use std::path::PathBuf;
use std::{env, fs};

type MenkutenToUnicodeMap = HashMap<(u8, u8, u8), String>;

/// JIS X 0213 マッピングテーブルをパースして面区点→Unicode変換テーブルを生成
fn generate_menkuten_map(
    table_path: &std::path::Path,
) -> Result<MenkutenToUnicodeMap, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(table_path)?;
    let mut map = MenkutenToUnicodeMap::new();

    for line in content.lines() {
        // コメント行と空行をスキップ
        if line.starts_with("##") || line.trim().is_empty() {
            continue;
        }

        // フォーマット: 3-XXXX\tU+YYYY\t# description
        // または: 4-XXXX\tU+YYYY+ZZZZ\t# description (合成文字)
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }

        let jis_code = parts[0];
        let unicode_str = parts[1];

        // JISコードをパース (例: "3-2421" -> plane=1, row=0x24=36, cell=0x21=33)
        // 注意: ファイル内では 3=plane1, 4=plane2
        let (plane, rowcell) = match jis_code.split_once('-') {
            Some(("3", rc)) => (1u8, rc),
            Some(("4", rc)) => (2u8, rc),
            _ => continue,
        };

        if rowcell.len() != 4 {
            continue;
        }

        let row = match u8::from_str_radix(&rowcell[0..2], 16) {
            Ok(r) => r.saturating_sub(0x20), // 0x21-0x7E -> 1-94
            Err(_) => continue,
        };
        let cell = match u8::from_str_radix(&rowcell[2..4], 16) {
            Ok(c) => c.saturating_sub(0x20), // 0x21-0x7E -> 1-94
            Err(_) => continue,
        };

        // Unicodeコードポイントをパース (U+XXXX または U+XXXX+YYYY)
        let unicode_chars = parse_unicode_sequence(unicode_str);
        if let Some(chars) = unicode_chars {
            map.insert((plane, row, cell), chars);
        }
    }

    Ok(map)
}

/// Unicode表記 (U+XXXX or U+XXXX+YYYY) をパースして文字列に変換
fn parse_unicode_sequence(s: &str) -> Option<String> {
    let mut result = String::new();

    for part in s.split('+') {
        let hex_str = part.trim_start_matches("U").trim_start_matches("u");
        if hex_str.is_empty() {
            continue;
        }

        let codepoint = u32::from_str_radix(hex_str, 16).ok()?;
        let c = char::from_u32(codepoint)?;
        result.push(c);
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let current: PathBuf = env::current_dir()?;

    let input_path = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| current.join("assets").join("jisx0213-2004-std.txt"));

    let output_path = args
        .get(2)
        .map(PathBuf::from)
        .unwrap_or_else(|| current.join("menkuten_to_unicode.map"));

    println!("JIS X 0213マッピングテーブルを解析中: {:?}", input_path);
    let menkuten_map = generate_menkuten_map(&input_path)?;

    println!("面区点→Unicode変換テーブルを保存中……");
    fs::write(&output_path, &rkyv::to_bytes::<_, 256>(&menkuten_map)?)?;

    println!("完了しました。");
    Ok(())
}
