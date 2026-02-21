use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::{AnalysedData, MapCache};

use aozora_rs_gaiji::gaiji_to_char;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::fmt::Write as FmtWrite;
use winnow::{
    Parser,
    combinator::{delimited, repeat, repeat_till},
    error::ContextError,
    token::{any, take_until},
};

fn analyse_file(path: &Path) -> Option<(usize, usize, String)> {
    let read_bin = std::fs::read(path).ok()?;
    let mut read_raw: &str = &encoding_rs::SHIFT_JIS.decode(&read_bin).0;
    let read = &mut read_raw;
    let gaiji_note = delimited("※［＃", take_until(0.., "］"), "］");
    let gaiji_notes = repeat(
        0..,
        repeat_till(0.., any::<_, ContextError>.void(), gaiji_note).map(|(_, s): ((), &str)| s),
    );

    let found = gaiji_notes
        .fold(
            || (0, 0, String::new()),
            |mut acc, mut e: &str| {
                if let Some(_) = gaiji_to_char(&mut e) {
                    acc.0 += 1;
                } else {
                    acc.1 += 1;
                    let _ = writeln!(acc.2, "{}", e);
                }
                acc
            },
        )
        .parse_next(read)
        .ok()?;
    Some(found)
}

pub async fn analyse_gaiji(
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

    for v in analysed {
        success += v.0;
        fail += v.1;
        if !v.2.is_empty() {
            write!(into, "{}", v.2)?;
        }
    }
    Ok(AnalysedData {
        success,
        fail,
        duration: start.elapsed(),
    })
}
