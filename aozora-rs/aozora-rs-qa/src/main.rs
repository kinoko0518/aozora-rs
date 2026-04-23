mod analyse;
mod map_cache;
mod sync;

pub const REPOSITORY: &str = "aozorabunko_text";

use std::path::PathBuf;
use std::time::Instant;
use std::{time::Duration};

pub use map_cache::{MapCache, update_map};
pub use sync::sync_repository;

use crate::analyse::analyse_works;

pub struct AnalysedSummary {
    pub success: usize,
    pub fail: usize,
    pub duration: Duration,
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

    let analyse_duration = Instant::now();
    println!("解析を実行中です……");
    analyse_works(manifest, &base_path, &map).await?;
    println!("解析が終了しました！（{:?}）", analyse_duration.elapsed());

    println!("すべて終了しました！");

    Ok(())
}
