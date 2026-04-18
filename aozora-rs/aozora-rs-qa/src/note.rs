use crate::{AnalysedSummary, MapCache};
use aozora_rs::internal::{AozoraTokenKind, parse_meta, tokenize, tokenizer::Note};
use aozora_rs::utf8tify_all_gaiji;
use rayon::prelude::*;
use std::collections::HashMap;
use std::io::Write as IoWrite;
use std::time::Instant;
use std::{
    fs::{self},
    path::{Path, PathBuf},
};
use winnow::LocatingSlice;

fn analyse_file(path: &Path) -> Option<(usize, usize, HashMap<String, usize>)> {
    let bytes = fs::read(path).ok()?;

    let read_original = encoding_rs::SHIFT_JIS.decode(&bytes).0;
    let read = utf8tify_all_gaiji(&read_original);

    let mut success = 0;
    let mut fail = 0;
    let mut failed_notes = HashMap::new();

    // メタデータは不要なので捨てる。消費後の body を tokenize に渡す
    let mut body = &read[..];
    let _ = parse_meta(&mut body);
    for token in tokenize(&mut LocatingSlice::new(body)).ok()? {
        if let AozoraTokenKind::Note(c) = &token.kind {
            match c {
                Note::Unknown(s) => {
                    fail += 1;
                    *failed_notes.entry(s.to_string()).or_insert(0) += 1;
                }
                _ => {
                    success += 1;
                }
            }
        }
    }

    Some((success, fail, failed_notes))
}

pub async fn note_analyse(
    into: &mut impl IoWrite,
    map: &MapCache,
) -> Result<AnalysedSummary, Box<dyn std::error::Error>> {
    let start = Instant::now();

    let (success, fail, merged_notes) = map
        .paths
        .par_iter()
        .filter_map(|path| analyse_file(&PathBuf::from(path)))
        .reduce(
            || (0, 0, HashMap::new()),
            |(mut s1, mut f1, mut map1), (s2, f2, map2)| {
                s1 += s2;
                f1 += f2;
                // 2つの HashMap を結合
                for (k, v) in map2 {
                    *map1.entry(k).or_insert(0) += v;
                }
                (s1, f1, map1)
            },
        );

    let mut sorted_notes: Vec<_> = merged_notes.into_iter().collect();
    sorted_notes.sort_unstable_by(|a, b| b.1.cmp(&a.1));

    for (note, count) in sorted_notes {
        writeln!(into, "{}: {}回", note, count)?;
    }

    Ok(AnalysedSummary {
        success,
        fail,
        duration: start.elapsed(),
    })
}
