mod gaiji;
mod map_cache;
mod note;
mod speed;
mod sync;

pub const REPOSITORY: &str = "aozorabunko_text";

use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;
use std::path::PathBuf;
use std::time::Instant;
use std::{fs::File, time::Duration};

pub use map_cache::{MapCache, update_map};
pub use sync::sync_repository;

use crate::speed::{SpeedSummary, speed_analyse};
use crate::{gaiji::analyse_gaiji, note::note_analyse};

pub struct AnalysedSummary {
    pub success: usize,
    pub fail: usize,
    pub duration: Duration,
}

impl AnalysedSummary {
    fn fancy(&self) -> String {
        format!(
            "{}件 | {}件 | {}% | {:?} |",
            self.success,
            self.fail,
            (self.success as f32) / ((self.success + self.fail) as f32) * 100.0,
            self.duration
        )
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let base_path = PathBuf::from(manifest);
    let az_base_path = PathBuf::from(format!("{}/{}", manifest, REPOSITORY));

    println!("aozora.rs品質保証プログラムへようこそ！");
    println!("最新の青空文庫へ同期しています……");
    if let Err(e) = sync::sync_repository(&base_path) {
        println!(
            "青空文庫へのアクセスに失敗しました。スキップして続行します……\n\t{}",
            e
        )
    };

    println!("マップを更新しています……");
    let map = map_cache::update_map(&base_path, &az_base_path)?;

    println!("ファイルを作成しています……");
    let mut note_log = File::create(format!("{}/result/invalid_note.txt", manifest))?;
    let mut gaiji_log = File::create(format!("{}/result/invalid_gaiji.txt", manifest))?;
    let mut speed_log = File::create(format!("{}/result/speed_report.md", manifest))?;
    let mut summary = String::new();

    let analyse_duration = Instant::now();
    println!("解析を実行中です……");
    let (note, gaiji, speed) = tokio::join!(
        note_analyse(&mut note_log, &map),
        analyse_gaiji(&mut gaiji_log, &map),
        speed_analyse(&mut speed_log, &base_path)
    );
    println!("解析が終了しました！（{:?}）", analyse_duration.elapsed());

    println!("調査報告書を作成中です……");
    writeln!(
        summary,
        "# QA結果\n| 解析種別 | 成功件数 | 失敗件数 | 成功率 | 処理時間 |\n| --- | --- | --- | --- | --- |"
    )?;
    write!(summary, "| 注記解析 | ")?;
    writeln!(summary, "{}", note?.fancy())?;
    write!(summary, "| 外字解析 | ")?;
    writeln!(summary, "{}", gaiji?.fancy())?;
    writeln!(
        summary,
        "\n# 速度レポート\n| タイトル | 処理時間 |\n| --- | --- |"
    )?;
    writeln!(
        summary,
        "{}",
        speed?
            .iter()
            .map(SpeedSummary::to_string)
            .collect::<Vec<String>>()
            .join("\n")
    )?;

    println!("{}", &summary);
    let mut summary_file = File::create(format!("{}/result/summary.md", manifest))?;
    write!(summary_file, "{}", &summary)?;

    println!("すべて終了しました！");

    Ok(())
}
