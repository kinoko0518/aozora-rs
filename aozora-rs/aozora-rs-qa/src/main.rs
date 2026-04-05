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
pub use sync::{GitSyncProgress, sync_repository, sync_repository_simple};

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
    let base_path = PathBuf::from("./aozora-rs-qa");
    let az_base_path = PathBuf::from(format!("./aozora-rs-qa/{}", REPOSITORY));

    println!("aozora.rs品質保証プログラムへようこそ！");
    println!("最新の青空文庫へ同期しています……");
    sync::sync_repository_simple(&base_path)?;

    println!("マップを更新しています……");
    let map = map_cache::update_map(&base_path, &az_base_path)?;

    println!("ファイルを作成しています……");
    let mut note_log = File::create("./aozora-rs-qa/result/invalid_note.txt")?;
    let mut gaiji_log = File::create("./aozora-rs-qa/result/invalid_gaiji.txt")?;
    let mut speed_log = File::create("./aozora-rs-qa/result/speed_report.md")?;
    let mut summary = String::new();

    let analyse_duration = Instant::now();
    println!("解析を実行中です……");
    let (note, gaiji, speed) = tokio::join!(
        note_analyse(&mut note_log, &map),
        analyse_gaiji(&mut gaiji_log, &map),
        speed_analyse(&mut speed_log)
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
    let mut summary_file = File::create("./aozora-rs-qa/result/summary.md")?;
    write!(summary_file, "{}", &summary)?;

    println!("すべて終了しました！");

    Ok(())
}
