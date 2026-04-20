use std::{
    cmp::Ordering,
    collections::HashMap,
    fs::File,
    io::Write,
    path::Path,
    time::{Duration, Instant},
};

use aozora_rs::{
    AozoraError, Dependencies,
    internal::{
        AozoraTokenKind, EpubSetting, Note, from_aozora_zip, parse_meta, retokenize,
        retokenized_to_xhtml, scopenize, tokenize,
    },
    utf8tify_all_gaiji,
};
use encoding_rs::SHIFT_JIS;
use plotters::prelude::*;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::{Either, ParallelIterator};
use serde::Serialize;
use winnow::LocatingSlice;

use crate::MapCache;

#[derive(Debug, Serialize, Clone)]
pub struct WorkAnalyse {
    // 作品メタデータ
    pub title: String,
    pub author: String,
    // 作品サイズ
    pub word_count: usize,
    pub byte_count: usize,
    pub token_count: usize,
    pub note_count: usize,
    // 変換エラー
    pub scopenize_errors: Vec<String>,
    pub retokenize_errors: Vec<String>,
    // 解析エラー
    pub invalid_gaiji: Vec<String>,
    pub invalid_notes: Vec<String>,
    // 各段階の所要時間
    pub read: Duration,
    pub gaiji_convert: Duration,
    pub get_meta: Duration,
    pub tokenize: Duration,
    pub scopenize: Duration,
    pub retokenize: Duration,
    pub xhtml_gen: Duration,
    pub epub_gen: Duration,
}

impl WorkAnalyse {
    pub fn pure_parsetime(&self) -> Duration {
        self.gaiji_convert + self.get_meta + self.tokenize + self.scopenize + self.retokenize
    }

    pub fn total_parsetime(&self) -> Duration {
        self.read
            + self.gaiji_convert
            + self.get_meta
            + self.tokenize
            + self.scopenize
            + self.retokenize
            + self.xhtml_gen
            + self.epub_gen
    }
}

fn analyse_per_work(s: &str, base_path: &Path) -> Result<WorkAnalyse, AozoraError> {
    let read_instant = Instant::now();
    let decode = |bytes: &[u8]| -> String {
        let (cow, _, _) = SHIFT_JIS.decode(bytes);
        cow.replace("\r\n", "\n")
    };
    let original_text = decode(&std::fs::read(s).map_err(|e| e.into())?);
    let read_duration = read_instant.elapsed();

    let gaiji_instant = Instant::now();
    let gaiji_converted = utf8tify_all_gaiji(original_text.as_str());
    let gaiji_duration = gaiji_instant.elapsed();
    let (s, invalid_gaijis) = gaiji_converted;
    let mut s_slice = s.as_ref();

    let meta_instant = Instant::now();
    let meta = parse_meta(&mut s_slice).map_err(|e| e.into())?;
    let meta_duration = meta_instant.elapsed();

    let title_owned = meta.title.to_string();
    let author_owned = meta.author.to_string();

    let tokenize_instant = Instant::now();
    let tokenized = tokenize(&mut LocatingSlice::new(s_slice)).map_err(|e| e.into())?;
    let tokenize_duration = tokenize_instant.elapsed();

    let invalid_notes: Vec<String> = tokenized
        .iter()
        .filter_map(|t| match &t.kind {
            AozoraTokenKind::Note(n) => match n {
                Note::Unknown(unknown) => Some(unknown),
                _ => None,
            },
            _ => None,
        })
        .map(|s| s.to_string())
        .collect();
    let note_count: usize = tokenized
        .iter()
        .filter_map(|s| match s.kind {
            AozoraTokenKind::Note(_) => Some(()),
            _ => None,
        })
        .count();
    let token_count: usize = tokenized.len();

    let scopenize_instant = Instant::now();
    let ((deco, flat), scopenize_errors) = scopenize(tokenized).into_tuple();
    let scopenized_duration = scopenize_instant.elapsed();

    let retokenize_instant = Instant::now();
    let (retokenized, retokenize_errors) = retokenize(flat, deco).into_tuple();
    let retokenized_duration = retokenize_instant.elapsed();

    let xhtmlnize_instant = Instant::now();
    let xhtmlnized = retokenized_to_xhtml(retokenized);
    let xhtmlnize_duration = xhtmlnize_instant.elapsed();

    let epub_instant = Instant::now();
    let epub_base_path = base_path.join("result/epubs");
    std::fs::create_dir_all(&epub_base_path).map_err(|e| e.into())?;
    from_aozora_zip(
        File::create(epub_base_path.join(format!("{}.epub", title_owned))).map_err(|e| e.into())?,
        &Dependencies::default(),
        &xhtmlnized,
        &EpubSetting {
            styles: vec![
                include_str!("../../aozora-rs/css/prelude.css"),
                include_str!("../../aozora-rs/css/vertical.css"),
                include_str!("../../../ayame/ayame/assets/miyabi.css"),
            ],
            ..Default::default()
        },
        &meta,
        &aozora_rs::PageInjectors::default(),
    )
    .map_err(|e| e.into())?;
    let epub_duration = epub_instant.elapsed();

    Ok(WorkAnalyse {
        title: title_owned,
        author: author_owned,

        word_count: s.chars().count(),
        note_count,
        token_count,
        byte_count: s.as_bytes().len(),

        scopenize_errors: scopenize_errors
            .iter()
            .map(|s| s.display(s_slice))
            .collect(),
        retokenize_errors: retokenize_errors.iter().map(|s| s.to_string()).collect(),
        invalid_gaiji: invalid_gaijis.iter().map(|s| s.to_string()).collect(),
        invalid_notes,

        read: read_duration,
        gaiji_convert: gaiji_duration,
        get_meta: meta_duration,
        tokenize: tokenize_duration,
        scopenize: scopenized_duration,
        retokenize: retokenized_duration,
        xhtml_gen: xhtmlnize_duration,
        epub_gen: epub_duration,
    })
}

const RANKING_LEN: usize = 10;

#[derive(Serialize)]
struct QASummary {
    // トータル処理時間
    total_duration: Duration,
    total_pure_parsetime: Duration,
    // ランキング
    duration_top: [WorkAnalyse; RANKING_LEN],
    pure_parsetime_top: [WorkAnalyse; RANKING_LEN],
    wordcount_top: [WorkAnalyse; RANKING_LEN],
    tokencount_top: [WorkAnalyse; RANKING_LEN],
    notecount_top: [WorkAnalyse; RANKING_LEN],
    // 処理速度毎秒
    bytes_per_sec: f32,
    wordcount_per_sec: f32,
    // 良品率
    fault_percent: f32,
    warning_perwork: f32,
}

pub async fn analyse_works(
    manifest: &str,
    base_path: &Path,
    path_map: &MapCache,
) -> Result<(), Box<dyn std::error::Error>> {
    let (ok_results, err_results): (HashMap<_, _>, HashMap<_, _>) = path_map
        .paths
        .par_iter()
        .map(|s| {
            analyse_per_work(s, base_path)
                .map(|o| (s.as_str(), o))
                .map_err(|e| (s.as_str(), e))
        })
        .partition_map(|res| match res {
            Ok(val) => Either::Left(val),
            Err((s, err)) => Either::Right((s, err.to_string())),
        });

    // JSON出力
    let mut succeed_log = File::create(format!("{}/result/succeed.json", manifest))?;
    let mut failed_log = File::create(format!("{}/result/failed.json", manifest))?;

    println!("JSONに解析を書き込み中です……");
    write!(
        &mut succeed_log,
        "{}",
        serde_json::to_string_pretty(&ok_results).unwrap()
    )?;
    write!(
        &mut failed_log,
        "{}",
        serde_json::to_string_pretty(&err_results).unwrap()
    )?;

    // スケールするようすをプロット
    enum XAxis {
        WordCount,
        NoteCount,
        TokenCount,
    }
    for x_axis in [XAxis::WordCount, XAxis::NoteCount, XAxis::TokenCount].iter() {
        println!(
            "{}",
            match x_axis {
                XAxis::WordCount => "文字数対処理時間のプロット図を作成中です……",
                XAxis::NoteCount => "注記数対処理時間のプロット図を作成中です……",
                XAxis::TokenCount => "トークン数対処理時間のプロット図を作成中です……",
            }
        );
        let path = base_path.join("result").join(match x_axis {
            XAxis::WordCount => "wordcount_vs_duration.png",
            XAxis::NoteCount => "notecount_vs_duration.png",
            XAxis::TokenCount => "tokencount_vs_duration.png",
        });
        let root = BitMapBackend::new(&path, (1920, 1080)).into_drawing_area();
        root.fill(&WHITE)?;

        let caption = match x_axis {
            XAxis::WordCount => "文字数に対する処理時間の増加",
            XAxis::NoteCount => "注記数に対する処理時間の増加",
            XAxis::TokenCount => "トークン数に対する処理時間の増加",
        };
        let mut chart = ChartBuilder::on(&root)
            .caption(caption, ("sans-serif", 30).into_font())
            .x_label_area_size(30)
            .y_label_area_size(40)
            .build_cartesian_2d(
                0f32..match x_axis {
                    XAxis::WordCount => 120_0000f32,
                    XAxis::NoteCount => 1_0000f32,
                    XAxis::TokenCount => 20_0000f32,
                },
                0f32..5f32,
            )?;

        chart.configure_mesh().draw()?;

        chart.draw_series(ok_results.iter().map(|(_, ok)| {
            Circle::new(
                (
                    match x_axis {
                        XAxis::WordCount => ok.word_count,
                        XAxis::NoteCount => ok.note_count,
                        XAxis::TokenCount => ok.token_count,
                    } as f32,
                    ok.pure_parsetime().as_secs_f32(),
                ),
                6,
                RED.mix(0.5).filled(),
            )
        }))?;

        root.present()?;
    }

    // サマリーを作成
    println!("サマリーを作成中……");
    let mut summary_file = File::create(format!("{}/result/summary.json", manifest))?;
    let mut ok_results: Vec<_> = ok_results.into_iter().map(|(_, a)| a).collect();

    let get_ranking_top = |sort_by: Box<dyn FnMut(&WorkAnalyse, &WorkAnalyse) -> Ordering>,
                           ok_results: &mut Vec<_>| {
        ok_results.sort_by(sort_by);
        std::array::from_fn(|i| ok_results[i].clone())
    };

    let duration_top: [WorkAnalyse; RANKING_LEN] = get_ranking_top(
        Box::new(|a, b| a.total_parsetime().cmp(&b.total_parsetime())),
        &mut ok_results,
    );
    let pure_parsetime_top: [WorkAnalyse; RANKING_LEN] = get_ranking_top(
        Box::new(|a, b| a.pure_parsetime().cmp(&b.pure_parsetime())),
        &mut ok_results,
    );
    let wordcount_top: [WorkAnalyse; RANKING_LEN] = get_ranking_top(
        Box::new(|a, b| a.word_count.cmp(&b.word_count)),
        &mut ok_results,
    );
    let tokencount_top: [WorkAnalyse; RANKING_LEN] = get_ranking_top(
        Box::new(|a, b| a.token_count.cmp(&b.token_count)),
        &mut ok_results,
    );
    let notecount_top: [WorkAnalyse; RANKING_LEN] = get_ranking_top(
        Box::new(|a, b| a.note_count.cmp(&b.note_count)),
        &mut ok_results,
    );

    let total_duration: Duration = ok_results.iter().map(|o| o.total_parsetime()).sum();
    let total_pure_parsetime: Duration = ok_results.iter().map(|o| o.pure_parsetime()).sum();

    let total_bytes: usize = ok_results.iter().map(|o| o.byte_count).sum();
    let bytes_per_sec: f32 = (total_bytes as f32) / total_duration.as_secs_f32();

    let total_wordcount: usize = ok_results.iter().map(|o| o.word_count).sum();
    let wordcount_per_sec: f32 = (total_wordcount as f32) / total_duration.as_secs_f32();

    let fault_percent: f32 = (err_results.iter().count() as f32) / (ok_results.len() as f32);
    let warning_total: usize = ok_results
        .iter()
        .map(|o| o.scopenize_errors.len() + o.retokenize_errors.len())
        .sum();
    let warning_perwork: f32 = (warning_total as f32) / (ok_results.len() as f32);

    let summary = QASummary {
        total_duration,
        total_pure_parsetime,

        duration_top,
        pure_parsetime_top,
        wordcount_top,
        tokencount_top,
        notecount_top,

        bytes_per_sec,
        wordcount_per_sec,

        fault_percent,
        warning_perwork,
    };
    writeln!(
        &mut summary_file,
        "{}",
        serde_json::to_string_pretty(&summary).unwrap()
    )?;

    Ok(())
}
