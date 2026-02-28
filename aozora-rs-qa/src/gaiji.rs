use std::collections::HashMap;
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::{AnalysedSummary, MapCache};

use aozora_rs_gaiji::gaiji_to_char;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use winnow::{
    Parser,
    combinator::{delimited, repeat, repeat_till},
    error::ContextError,
    token::{any, take_until},
};

fn analyse_file(path: &Path) -> Option<(usize, usize, HashMap<String, usize>)> {
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
            || (0, 0, HashMap::new()),
            |mut acc, e: &str| {
                if gaiji_to_char(e).is_some() {
                    acc.0 += 1;
                } else {
                    acc.1 += 1;
                    // 解析に失敗した文字列をキーにしてカウントアップ
                    *acc.2.entry(e.to_string()).or_insert(0) += 1;
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
) -> Result<AnalysedSummary, Box<dyn std::error::Error>> {
    let start = Instant::now();

    // rayon の reduce を使い、各スレッドの HashMap とカウントを並列にマージ
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

    // HashMap を Vec に変換し、失敗回数の降順でソート
    let mut sorted_notes: Vec<_> = merged_notes.into_iter().collect();
    sorted_notes.sort_unstable_by(|a, b| b.1.cmp(&a.1));

    // 結果の書き込み
    for (note, count) in sorted_notes {
        writeln!(into, "{}: {}回", note, count)?;
    }

    Ok(AnalysedSummary {
        success,
        fail,
        duration: start.elapsed(),
    })
}
