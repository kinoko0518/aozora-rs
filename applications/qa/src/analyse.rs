mod markdown;
mod metrics;
mod per_work;
mod plot;
mod sysinfo;

use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    time::Instant,
};

use const_format::concatcp;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::{Either, ParallelIterator};

pub use markdown::write_markdown_report;
pub use metrics::compute_performance_report;
pub use per_work::{WorkAnalyse, analyse_per_work};

use crate::{
    MapCache, RESULT_OUT_PATH,
    analyse::{
        plot::{XAxis, plot_result},
        sysinfo::get_sysinfo,
    },
};

pub const REPORT_MD_PATH: &str = concatcp!(RESULT_OUT_PATH, "/report.md");

pub fn write_to_json(
    ok_results: &HashMap<&str, WorkAnalyse>,
    err_results: &HashMap<&str, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(RESULT_OUT_PATH)?;

    let mut succeed_log = File::create(concatcp!(RESULT_OUT_PATH, "/succeed.json"))?;
    let mut failed_log = File::create(concatcp!(RESULT_OUT_PATH, "/failed.json"))?;

    write!(
        &mut succeed_log,
        "{}",
        serde_json::to_string_pretty(&ok_results)?
    )?;
    write!(
        &mut failed_log,
        "{}",
        serde_json::to_string_pretty(&err_results)?
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

    // 代表作の抽出（存在する場合）
    let remarkables: HashMap<&str, &WorkAnalyse> = ok_results
        .iter()
        .filter(|(_, v)| ["罪と罰", "春と修羅", "桜桃"].contains(&v.title.as_str()))
        .map(|(_, v)| (v.title.as_str(), v))
        .collect();

    if !remarkables.is_empty() {
        let mut remarkables_json = File::create(concatcp!(RESULT_OUT_PATH, "/remarkable.json"))?;
        writeln!(
            &mut remarkables_json,
            "{}",
            serde_json::to_string_pretty(&remarkables)?
        )?;
    }

    // プロット図を描画（成功数が1件以上ある場合のみ）
    if !ok_results.is_empty() {
        for x_axis in [XAxis::WordCount, XAxis::TokenCount, XAxis::DecoCount].iter() {
            println!(
                "{}",
                match x_axis {
                    XAxis::WordCount => "文字数対処理時間のプロット図を作成中です……",
                    XAxis::DecoCount => "注記数対処理時間のプロット図を作成中です……",
                    XAxis::TokenCount => "トークン数対処理時間のプロット図を作成中です……",
                }
            );
            if let Err(e) = plot_result(x_axis, &ok_results) {
                println!("プロット作成をスキップしました: {}", e);
            }
        }
    }

    // サマリーおよびパフォーマンスレポートを作成
    println!("サマリーを作成中……");
    let ok_results_vec: Vec<WorkAnalyse> = ok_results.into_values().collect();

    let report = compute_performance_report(
        &ok_results_vec,
        path_map.paths.len(),
        err_results.len(),
        total_elapsed,
    );

    let mut summary_file = File::create(concatcp!(RESULT_OUT_PATH, "/summary.json"))?;
    writeln!(
        &mut summary_file,
        "{}",
        serde_json::to_string_pretty(&report)?
    )?;

    // 実行環境の取得
    println!("実行環境を取得しています……");
    let mut enviroment_file = File::create(concatcp!(RESULT_OUT_PATH, "/enviroment.json"))?;
    write!(
        &mut enviroment_file,
        "{}",
        serde_json::to_string(&get_sysinfo())?
    )?;

    // Markdownレポートの出力
    write_markdown_report(&report, REPORT_MD_PATH)?;
    println!("レポートを出力しました: {}", REPORT_MD_PATH);

    Ok(())
}
