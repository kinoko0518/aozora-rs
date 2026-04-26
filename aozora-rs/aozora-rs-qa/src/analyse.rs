mod per_work;
mod plot;

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
use sysinfo::{Disks, System};

use crate::{
    MapCache, RESULT_OUT_PATH,
    analyse::plot::{XAxis, plot_result},
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

#[derive(Serialize)]
struct SysInfo {
    os_name: String,
    os_version: String,
    kernel: String,
    architecture: String,

    cpu_name: String,
    memory_size: u64,
    disk_info: Vec<(u64, String, String)>,

    rustc_version: String,
}

fn get_sysinfo() -> SysInfo {
    let mut sys = System::new_all();
    sys.refresh_all();

    // OS情報
    let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
    let os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());
    let kernel = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
    let architecture = System::cpu_arch();

    // CPU情報
    let cpus = sys.cpus();
    let cpu_name = if let Some(cpu) = cpus.first() {
        cpu.brand()
    } else {
        "None"
    }
    .to_string();

    // メモリ情報
    let memory_size = sys.total_memory();

    // ドライブ情報
    let disks = Disks::new_with_refreshed_list();
    let disk_info: Vec<_> = disks
        .list()
        .iter()
        .map(|disk| {
            let size_gb = disk.total_space();
            let mount_point = disk.mount_point().to_string_lossy().to_string();
            let fs_type = disk.file_system().to_string_lossy().to_string();

            (size_gb, mount_point, fs_type)
        })
        .collect();

    let rustc_version = env!("RUSTC_VERSION").to_string();

    SysInfo {
        os_name,
        os_version,
        kernel,
        architecture,
        cpu_name,
        memory_size,
        disk_info,
        rustc_version,
    }
}

pub async fn analyse_works(path_map: &MapCache) -> Result<(), Box<dyn std::error::Error>> {
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
    let mut ok_results: Vec<_> = ok_results.into_iter().map(|(_, a)| a).collect();

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
        Box::new(|a, b| a.total_parsetime().cmp(&b.total_parsetime()).reverse()),
        &mut ok_results,
    );

    let total_duration: Duration = ok_results.iter().map(|o| o.total_parsetime()).sum();

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
