mod gaiji;
mod map_cache;
mod note;
mod sync;

pub const REPOSITORY: &str = "aozorabunko_text";

use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;
use std::path::PathBuf;
use std::time::Instant;
use std::{fs::File, time::Duration};

pub use map_cache::{MapCache, update_map};
pub use sync::{GitSyncProgress, sync_repository, sync_repository_simple};

use crate::{gaiji::analyse_gaiji, note::note_analyse};

pub struct AnalysedData {
    pub success: usize,
    pub fail: usize,
    pub duration: Duration,
}

impl AnalysedData {
    fn fancy(&self) -> String {
        format!(
            "\t成功: {}\n\t失敗: {}\n\t成功率: {}%\n\t処理時間: {:?}",
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
    let mut summary = String::new();

    let analyse_duration = Instant::now();
    println!("解析を実行中です……");
    let (note, gaiji) = tokio::join!(
        note_analyse(&mut note_log, &map),
        analyse_gaiji(&mut gaiji_log, &map)
    );
    println!("解析が終了しました！（{:?}）", analyse_duration.elapsed());

    println!("調査報告書を作成中です……");
    writeln!(summary, "==== aozora-rs　品質保証プログラム ====")?;
    writeln!(summary, "\n注記解析結果：")?;
    writeln!(
        summary,
        "{}",
        match note {
            Ok(n) => n.fancy(),
            Err(e) => e.to_string(),
        }
    )?;
    writeln!(summary, "\n外字解析結果：")?;
    writeln!(
        summary,
        "{}",
        match gaiji {
            Ok(n) => n.fancy(),
            Err(e) => e.to_string(),
        }
    )?;

    println!("{}", &summary);
    let mut summary_file = File::create("./aozora-rs-qa/result/summary.md")?;
    write!(summary_file, "{}", &summary)?;

    println!("すべて終了しました！");

    Ok(())
}
