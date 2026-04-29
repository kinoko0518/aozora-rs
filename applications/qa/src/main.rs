mod analyse;
mod map_cache;
mod sync;

pub const MANIFEST: &str = env!("CARGO_MANIFEST_DIR");
pub const REPOSITORY: &str = "aozorabunko_text";

pub const AOZORABUNKO_TEXT_PATH: &str = concatcp!(MANIFEST, "/assets/", REPOSITORY);
pub const EPUB_OUT_PATH: &str = concatcp!(MANIFEST, "/out/epubs");
pub const RESULT_OUT_PATH: &str = concatcp!(MANIFEST, "/out/result");

pub const CACHE_BIN_PATH: &str = concatcp!(MANIFEST, "/cache.bin");

use const_format::concatcp;

use std::time::Duration;
use std::time::Instant;

pub use map_cache::{MapCache, update_map};
pub use sync::sync_repository;

use crate::analyse::analyse_all_works;

pub struct AnalysedSummary {
    pub success: usize,
    pub fail: usize,
    pub duration: Duration,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("aozora.rs品質保証プログラムへようこそ！");
    println!("最新の青空文庫へ同期しています……");
    if let Err(e) = sync::sync_repository(AOZORABUNKO_TEXT_PATH) {
        println!(
            "青空文庫へのアクセスに失敗しました。スキップして続行します……\n\t{}",
            e
        )
    };

    println!("マップを更新しています……");
    let map = map_cache::update_map(CACHE_BIN_PATH, AOZORABUNKO_TEXT_PATH)?;

    let analyse_duration = Instant::now();
    println!("全量解析を実行中です……");
    analyse_all_works(&map).await?;
    println!(
        "全量解析が終了しました！（{:?}）",
        analyse_duration.elapsed()
    );

    println!("すべて終了しました！");

    Ok(())
}
