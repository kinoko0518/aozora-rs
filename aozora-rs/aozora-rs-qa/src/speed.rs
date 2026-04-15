use std::{
    fs::File,
    io::Write,
    path::Path,
    time::{Duration, Instant},
};

use aozora_rs_core::{parse_meta, retokenize, scopenize, tokenize};
use aozora_rs_epub::EpubSetting;
use aozora_rs_zip::Dependencies;
use encoding_rs::SHIFT_JIS;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use winnow::LocatingSlice;

#[derive(Debug)]
pub struct SpeedPerWork {
    pub title: String,
    pub author: String,
    pub gaiji_convert: Duration,
    pub get_meta: Duration,
    pub tokenize: Duration,
    pub scopenize: Duration,
    pub retokenize: Duration,
    pub xhtml_gen: Duration,
    pub epub_gen: Duration,
}

impl SpeedPerWork {
    pub fn fancy(&self) -> String {
        let total = self.total();
        let pure_parse_total = total - self.epub_gen;
        format!(
            "## {} - {}\n| 実行項目 | 処理時間 |\n| --- | --- |\n| 全体処理時間 | {:?} |\n| 純粋変換時間 | {:?}（{}%） |\n| 外字変換 | {:?} |\n| メタデータ解析 | {:?} |\n| トークン化 | {:?} |\n| スコープ化 | {:?} |\n| 再トークン化 | {:?} |\n| XHTML生成 | {:?} |\n| epub生成 | {:?} |",
            self.title,
            self.author,
            total,
            pure_parse_total,
            (pure_parse_total.as_secs_f32() / total.as_secs_f32()) * 100.,
            self.gaiji_convert,
            self.get_meta,
            self.tokenize,
            self.scopenize,
            self.retokenize,
            self.xhtml_gen,
            self.epub_gen
        )
    }

    pub fn total(&self) -> Duration {
        self.epub_gen
            + self.gaiji_convert
            + self.get_meta
            + self.retokenize
            + self.scopenize
            + self.tokenize
            + self.xhtml_gen
    }
}

pub struct SpeedSummary {
    title: String,
    duration: Duration,
}

impl std::fmt::Display for SpeedSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "| {} | {:?} |", self.title, self.duration)
    }
}

fn analyse_per_work(
    s: String,
    base_path: &Path,
) -> Result<SpeedPerWork, Box<dyn std::error::Error>> {
    let gaiji_instant = Instant::now();
    let gaiji_converted = aozora_rs_gaiji::whole_gaiji_to_char(s.as_str());
    let gaiji_duration = gaiji_instant.elapsed();
    let mut s_slice: &str = &gaiji_converted;

    let meta_instant = Instant::now();
    let meta = parse_meta(&mut s_slice).map_err(|e| e.to_string())?;
    let meta_duration = meta_instant.elapsed();

    let title_owned = meta.title.to_string();
    let author_owned = meta.author.to_string();

    let tokenize_instant = Instant::now();
    let tokenized = tokenize(&mut LocatingSlice::new(s_slice)).map_err(|e| e.to_string())?;
    let tokenize_duration = tokenize_instant.elapsed();

    let scopenize_instant = Instant::now();
    let ((deco, flat), mut errors) = scopenize(tokenized, s_slice).into_tuple();
    let scopenized_duration = scopenize_instant.elapsed();

    let retokenize_instant = Instant::now();
    let (retokenized, retokenized_errors) = retokenize(flat, deco).into_tuple();
    errors.extend(retokenized_errors.into_iter());
    let retokenized_duration = retokenize_instant.elapsed();

    let xhtmlnize_instant = Instant::now();
    let xhtmlnized = aozora_rs_xhtml::retokenized_to_novel_result(retokenized, meta, errors);
    let xhtmlnize_duration = xhtmlnize_instant.elapsed();

    let epub_instant = Instant::now();
    let epub_base_path = base_path.join("result/epubs");
    std::fs::create_dir_all(&epub_base_path)?;
    let _ = aozora_rs_epub::from_aozora_zip(
        File::create(epub_base_path.join(format!("{}.epub", title_owned)))?,
        Dependencies::default(),
        EpubSetting {
            styles: vec![
                include_str!("../../../ayame/ayame/assets/prelude.css"),
                include_str!("../../../ayame/ayame/assets/vertical.css"),
                include_str!("../../../ayame/ayame/assets/miyabi.css"),
            ],
            ..Default::default()
        },
        xhtmlnized,
    )?;
    let epub_duration = epub_instant.elapsed();

    Ok(SpeedPerWork {
        title: title_owned,
        author: author_owned,
        gaiji_convert: gaiji_duration,
        get_meta: meta_duration,
        tokenize: tokenize_duration,
        scopenize: scopenized_duration,
        retokenize: retokenized_duration,
        xhtml_gen: xhtmlnize_duration,
        epub_gen: epub_duration,
    })
}

pub async fn speed_analyse(
    log: &mut File,
    base_path: &Path,
) -> Result<Vec<SpeedSummary>, Box<dyn std::error::Error>> {
    let decode = |bytes: &[u8]| -> String {
        let (cow, _, _) = SHIFT_JIS.decode(bytes);
        cow.replace("\r\n", "\n")
    };

    let works = [
        decode(include_bytes!("../../example/haruto_shura.txt")),
        decode(include_bytes!("../../example/oto.txt")),
        decode(include_bytes!("../../example/shayo.txt")),
        decode(include_bytes!("../../example/tsumito_batsu.txt")),
    ];

    let results: Result<Vec<SpeedPerWork>, _> = works
        .into_par_iter()
        .map(|s| analyse_per_work(s, base_path).map_err(|e| e.to_string()))
        .collect();

    let vec_results = results?;
    let arr: [SpeedPerWork; 4] = vec_results.try_into().unwrap();

    writeln!(log, "# 処理時間レポート")?;
    for s in &arr {
        writeln!(log, "{}", s.fancy())?;
    }

    Ok(arr
        .map(|a| SpeedSummary {
            duration: a.total(),
            title: a.title,
        })
        .into_iter()
        .collect())
}
