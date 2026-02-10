use crate::{AnalysedData, MapCache};
use aozora_rs_core::prelude::{AozoraTokenKind, Note};
use aozora_rs_core::tokenize;
use aozora_rs_gaiji::whole_gaiji_to_char;
use rayon::prelude::*;
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;
use std::time::Instant;
use std::{
    fs::{self},
    path::{Path, PathBuf},
};

fn analyse_file(path: &Path) -> Option<(usize, usize, String)> {
    let bytes = fs::read(path).ok()?;

    let read_original = encoding_rs::SHIFT_JIS.decode(&bytes).0;
    let read = whole_gaiji_to_char(&read_original);

    let mut success = 0;
    let mut fail = 0;
    let mut failed_notes = String::new();

    for token in tokenize(&read).map(|(_, tokens)| tokens).ok()? {
        match &token.kind {
            AozoraTokenKind::Note(c) => match c {
                Note::Unknown(s) => {
                    fail += 1;
                    writeln!(failed_notes, "{}", s);
                }
                _ => {
                    success += 1;
                }
            },
            _ => {}
        }
    }

    Some((success, fail, failed_notes))
}

pub async fn note_analyse(
    into: &mut impl IoWrite,
    map: &MapCache,
) -> Result<AnalysedData, Box<dyn std::error::Error>> {
    let start = Instant::now();

    let analysed = map
        .paths
        .par_iter()
        .filter_map(|path| analyse_file(&PathBuf::from(path)))
        .collect::<Vec<(usize, usize, String)>>();

    let mut success = 0;
    let mut fail = 0;

    for f in analysed {
        success += f.0;
        fail += f.1;
        if !f.2.is_empty() {
            write!(into, "{}", f.2)?;
        }
    }

    Ok(AnalysedData {
        success,
        fail,
        duration: start.elapsed(),
    })
}
