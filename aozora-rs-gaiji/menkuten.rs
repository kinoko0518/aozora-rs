use gaiji_chuki_parser::Menkuten;
use gaiji_chuki_parser::unicode;
use rkyv::rancor::Error;
use rkyv::to_bytes;
use rkyv::util::AlignedVec;
use std::fs::File;
use std::hash::Hash;
use std::hash::Hasher;
use std::io::Write;
use std::{collections::HashMap, hash::DefaultHasher, path::Path};
use winnow::{Parser, ascii::*, combinator::*, error::ContextError, token::any};

use crate::ignore_rest_of_line;

pub type MenkutenTable = HashMap<Menkuten, String>;

/// 1E-F0C5のような文字列にマッチし、面句点として数字を分解する
fn parse_single_menkuten(input: &mut &str) -> Result<Menkuten, ContextError> {
    (hex_digit1, '-', hex_digit1)
        .map(|(plane, _, row_and_cell): (&str, _, &str)| {
            let plane = u8::from_str_radix(plane, 16).unwrap();
            let row_and_cell = u32::from_str_radix(row_and_cell, 16).unwrap();
            (
                plane,
                (row_and_cell >> 8) as u8,
                (row_and_cell & 0xFF) as u8,
            )
        })
        .parse_next(input)
}

fn parse_line<'s>(input: &mut &'s str) -> Result<Option<(Menkuten, String)>, ContextError> {
    let data_line =
        (parse_single_menkuten, space1, unicode).map(|(menkuten, _, utf8)| Some((menkuten, utf8)));

    let comment_or_empty = ignore_rest_of_line.map(|_| None);

    alt((data_line, comment_or_empty)).parse_next(input)
}

fn parse_menkuten<'s>(input: &mut &'s str) -> MenkutenTable {
    input
        .lines()
        .into_iter()
        .filter_map(|mut line| {
            (
                opt((parse_line, space1).map(|(one, _)| one)),
                "#",
                repeat(0.., any).map(|_: ()| ()),
            )
                .map(|(one, _, _): (_, _, ())| one.flatten())
                .parse_next(&mut line)
                .ok()
                .flatten()
        })
        .collect()
}

pub async fn satisfy_latest_menkuten(
    out_dir: &Path,
) -> Result<MenkutenTable, Box<dyn std::error::Error>> {
    let menkuten: String = reqwest::get("http://x0213.org/codetable/jisx0213-2004-std.txt")
        .await?
        .text()
        .await?;

    let txt_path = out_dir.join("jisx0213-2004-std.txt");
    let map_path = out_dir.join("menkuten_to_unicode.map");

    let read_hash = std::fs::read_to_string(&txt_path)
        .map(|read| {
            let mut hasher = DefaultHasher::new();
            read.hash(&mut hasher);
            hasher.finish()
        })
        .unwrap_or(0);

    let latest_hash = {
        let mut hasher = DefaultHasher::new();
        menkuten.hash(&mut hasher);
        hasher.finish()
    };

    if read_hash != latest_hash || !map_path.exists() {
        let mut menkuten_file = File::create(&txt_path)?;
        write!(menkuten_file, "{}", menkuten)?;

        let map_data = parse_menkuten(&mut menkuten.as_str());

        let mut menkuten_map = File::create(&map_path)?;
        let bytes = to_bytes::<Error>(&map_data)?;
        menkuten_map.write_all(&bytes)?;
    }

    let bytes = std::fs::read(&map_path)?;
    let mut aligned = AlignedVec::<16>::with_capacity(bytes.len());
    aligned.extend_from_slice(&bytes);

    Ok(
        rkyv::from_bytes::<MenkutenTable, rkyv::rancor::Error>(&aligned)
            .expect("menkuten_to_unicode.map data is corrupted"),
    )
}
