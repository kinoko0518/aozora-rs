mod analyse;
mod map_cache;
mod sync;

use std::path::Path;
use std::time::Instant;

use const_format::concatcp;

pub const MANIFEST: &str = env!("CARGO_MANIFEST_DIR");
pub const REPOSITORY: &str = "aozorabunko_text";

pub const AOZORABUNKO_TEXT_PATH: &str = concatcp!(MANIFEST, "/assets/", REPOSITORY);
pub const EPUB_OUT_PATH: &str = concatcp!(MANIFEST, "/out/epubs");
pub const RESULT_OUT_PATH: &str = concatcp!(MANIFEST, "/out/result");
pub const CACHE_BIN_PATH: &str = concatcp!(MANIFEST, "/cache.bin");

pub use map_cache::{MapCache, update_map};
pub use sync::sync_repository;

use crate::analyse::analyse_all_works;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("aozora.rs 品質保証 & パフォーマンスプロファイリングプログラム");

    let args: Vec<String> = std::env::args().collect();
    let (target_path, is_custom_path) = if args.len() > 1 {
        let path = args[1].clone();
        println!("指定されたパスを解析対象にします: {}", path);
        (path, true)
    } else {
        println!("デフォルトパスを対象にします: {}", AOZORABUNKO_TEXT_PATH);
        (AOZORABUNKO_TEXT_PATH.to_string(), false)
    };

    // デフォルトパスかつ未存在の場合のみリモートと同期
    if !is_custom_path && !Path::new(&target_path).exists() {
        println!("最新の青空文庫へ同期しています……");
        if let Err(e) = sync::sync_repository(&target_path) {
            println!(
                "青空文庫へのアクセスに失敗しました。スキップして続行します……\n\t{}",
                e
            );
        }
    }

    println!("解析対象ファイルをスキャンしています……");
    let map = if is_custom_path {
        MapCache::generate_map(&target_path)?
    } else {
        map_cache::update_map(CACHE_BIN_PATH, &target_path)?
    };

    if map.paths.is_empty() {
        println!(
            "警告: 解析対象のテキストファイルが見つかりませんでした: {}",
            target_path
        );
        println!("ヒント: 引数に対象ディレクトリまたは.txtファイルを指定できます:");
        println!("  cargo run --release -p aozora-rs-qa -- <path_to_dir_or_file>");
        return Ok(());
    }

    println!("{} 件の作品を検出しました。並列解析を実行します……", map.paths.len());

    let analyse_duration = Instant::now();
    analyse_all_works(&map).await?;
    println!(
        "全量解析が終了しました（所要時間: {:?}）",
        analyse_duration.elapsed()
    );

    Ok(())
}
