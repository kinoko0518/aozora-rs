mod per_work;
mod plot;
mod sysinfo;

use std::{
    cmp::Ordering,
    collections::HashMap,
    fs::File,
    io::Write,
    time::{Duration, Instant},
};

use const_format::concatcp;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::{Either, ParallelIterator};
use serde::Serialize;

pub use per_work::{WorkAnalyse, analyse_per_work};

use crate::{
    MapCache, RESULT_OUT_PATH,
    analyse::{
        plot::{XAxis, plot_result},
        sysinfo::get_sysinfo,
    },
};

const RANKING_LEN: usize = 10;

#[derive(Serialize)]
struct QASummary {
    // 統計
    total_works: usize,
    total_succeed: usize,
    total_failed: usize,
    scopenize_warning_total: usize,
    retokenize_warning_total: usize,
    total_bytes: usize,
    total_wordcount: usize,
    // トータル処理時間
    duration_every_thread_total: Duration,
    duration_total: Duration,
    // ランキング
    duration_top: [WorkAnalyse; RANKING_LEN],
}

pub fn write_to_json(
    ok_results: &HashMap<&str, WorkAnalyse>,
    err_results: &HashMap<&str, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(RESULT_OUT_PATH)?;
    // JSON出力
    let mut succeed_log = File::create(concatcp!(RESULT_OUT_PATH, "/succeed.json"))?;
    let mut failed_log = File::create(concatcp!(RESULT_OUT_PATH, "/failed.json"))?;

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

    Ok(())
}

pub async fn analyse_all_works(path_map: &MapCache) -> Result<(), Box<dyn std::error::Error>> {
    let total_duration = Instant::now();
    let (ok_results, err_results): (HashMap<_, _>, HashMap<_, _>) = path_map
        .paths
        .par_iter()
        .map(|s| {
            analyse_per_work(s)
                .map(|o| (s.as_str(), o))
                .map_err(|e| (s.as_str(), e))
        })
        .partition_map(|res| match res {
            Ok(val) => Either::Left(val),
            Err((s, err)) => Either::Right((s, err.to_string())),
        });

    let total_elapsed = total_duration.elapsed();

    // JSONに結果を書き出し
    println!("JSONに解析を書き込み中です……");
    write_to_json(&ok_results, &err_results)?;

    // 例をJSONに書き出し
    let remarkables: HashMap<&str, &WorkAnalyse> = ok_results
        .iter()
        .filter(|(_, v)| ["罪と罰", "春と修羅", "桜桃"].contains(&v.title.as_str()))
        .map(|(_, v)| (v.title.as_str(), v))
        .collect();

    let mut remarkables_json = File::create(concatcp!(RESULT_OUT_PATH, "/remarkable.json"))?;
    writeln!(
        &mut remarkables_json,
        "{}",
        serde_json::to_string_pretty(&remarkables)?
    )?;

    // プロット図を描画
    for x_axis in [XAxis::WordCount, XAxis::TokenCount, XAxis::DecoCount].iter() {
        println!(
            "{}",
            match x_axis {
                XAxis::WordCount => "文字数対処理時間のプロット図を作成中です……",
                XAxis::DecoCount => "注記数対処理時間のプロット図を作成中です……",
                XAxis::TokenCount => "トークン数対処理時間のプロット図を作成中です……",
            }
        );
        plot_result(x_axis, &ok_results)?;
    }

    // サマリーを作成
    println!("サマリーを作成中……");
    let mut summary_file = File::create(concatcp!(RESULT_OUT_PATH, "/summary.json"))?;
    let mut ok_results: Vec<_> = ok_results.into_values().collect();

    let get_ranking_top = |sort_by: Box<dyn FnMut(&WorkAnalyse, &WorkAnalyse) -> Ordering>,
                           ok_results: &mut Vec<_>| {
        ok_results.sort_by(sort_by);
        std::array::from_fn(|i| {
            if i < ok_results.len() {
                ok_results[i].clone()
            } else {
                WorkAnalyse::default()
            }
        })
    };

    let duration_top: [WorkAnalyse; RANKING_LEN] = get_ranking_top(
        Box::new(|a, b| a.total_parsetime.cmp(&b.total_parsetime).reverse()),
        &mut ok_results,
    );

    let total_duration: Duration = ok_results.iter().map(|o| o.total_parsetime).sum();

    let total_bytes: usize = ok_results.iter().map(|o| o.byte_count).sum();
    let total_wordcount: usize = ok_results.iter().map(|o| o.word_count).sum();

    let scopenize_warning_total: usize = ok_results.iter().map(|o| o.scopenize_errors.len()).sum();
    let retokenize_warning_total: usize =
        ok_results.iter().map(|o| o.retokenize_errors.len()).sum();

    let summary = QASummary {
        total_works: path_map.paths.len(),
        total_succeed: ok_results.len(),
        total_failed: err_results.len(),
        scopenize_warning_total,
        retokenize_warning_total,

        total_bytes,
        total_wordcount,

        duration_every_thread_total: total_duration,
        duration_total: total_elapsed,

        duration_top,
    };
    writeln!(
        &mut summary_file,
        "{}",
        serde_json::to_string_pretty(&summary).unwrap()
    )?;

    // 実行環境の取得
    println!("実行環境を取得しています……");
    let mut enviroment_file = File::create(concatcp!(RESULT_OUT_PATH, "/enviroment.json"))?;
    write!(
        &mut enviroment_file,
        "{}",
        serde_json::to_string(&get_sysinfo()).unwrap()
    )?;

    Ok(())
}
