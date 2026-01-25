use aozora_rs::{
    prelude::{AozoraTokenKind, tokenize},
    tokenizer::prelude::Note,
};
use aozora_rs_gaiji::whole_gaiji_to_char;
use aozora_rs_tester::{AnalysedData, update_map};
use rayon::prelude::*;
use std::io::Write;
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

fn analyse_file(path: &Path) -> Option<AnalysedData> {
    let bytes = fs::read(path).ok()?;

    let read_original = encoding_rs::SHIFT_JIS.decode(&bytes).0;
    let read = whole_gaiji_to_char(&read_original);

    let result = tokenize(&mut winnow::LocatingSlice::new(&read))
        .ok()?
        .iter()
        .fold(AnalysedData::new(), |mut acc, token| {
            match &token.kind {
                AozoraTokenKind::Command(c) => match c {
                    Note::Unknown(s) => acc.fail(s.to_string()),
                    _ => acc.success(),
                },
                _ => {}
            }
            acc
        });
    Some(result)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("注記解析を開始します……");

    let map = update_map(&std::env::current_dir()?)?;

    let result = map
        .paths
        .par_iter()
        .map(|path| analyse_file(&PathBuf::from(path)))
        .filter_map(|s| s)
        .reduce(AnalysedData::new, |acc, d| acc.join(d));

    println!("\n----------- 解析完了！ -----------\n");

    let mut file = File::create("./invalid_note_report.txt").unwrap();

    let failed_len = result.failed.iter().fold(0_usize, |acc, v| {
        writeln!(file, "{}", v).unwrap();
        acc + 1
    });

    let total = result.successed + failed_len;
    let rate = if total > 0 {
        (result.successed as f32) / (total as f32) * 100.0
    } else {
        0.0
    };

    println!(
        "\n成功：{}件 失敗{}件 成功率{}%",
        result.successed, failed_len, rate
    );
    Ok(())
}
