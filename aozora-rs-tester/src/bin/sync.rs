use aozora_rs_tester::{
    GitSyncProgress, MapCacheProgress, sync_repository, update_map_with_progress,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let current = std::env::current_dir()?;

    println!("青空文庫の最新版と同期中……");
    sync_repository(&current, |progress| match progress {
        GitSyncProgress::Checking => println!("  リポジトリを確認中..."),
        GitSyncProgress::Cloning => println!("  クローン中..."),
        GitSyncProgress::Pulling => println!("  最新版を取得中..."),
        GitSyncProgress::Done => println!("  同期完了"),
        GitSyncProgress::Error(e) => eprintln!("  エラー: {}", e),
    })?;

    println!("\nマップを更新します……");
    update_map_with_progress(&current, |progress| match progress {
        MapCacheProgress::CheckingCache => println!("  キャッシュを確認中..."),
        MapCacheProgress::CacheFound => println!("  キャッシュが見つかりました"),
        MapCacheProgress::CacheOutdated => println!("  キャッシュが古いです"),
        MapCacheProgress::CacheUpToDate => println!("  キャッシュは最新です"),
        MapCacheProgress::CacheNotFound => println!("  キャッシュが見つかりません"),
        MapCacheProgress::GeneratingMap => println!("  マップを生成中..."),
        MapCacheProgress::SavingCache => println!("  キャッシュを保存中..."),
        MapCacheProgress::Done => println!("  完了"),
    })?;

    println!("\n同期が完了しました！");
    Ok(())
}
