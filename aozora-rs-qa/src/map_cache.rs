use rkyv::{Archive, Deserialize, Serialize, archived_root};
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};
use walkdir::WalkDir;

// MapCacheProgress と crate::REPOSITORY の依存を削除

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[archive_attr(derive(Debug))]
pub struct MapCache {
    id: String,
    pub paths: Vec<String>,
}

impl MapCache {
    /// 指定されたリポジトリルートで git rev-parse HEAD を実行してコミットIDを取得
    fn get_head_commit_id(repo_root: &Path) -> Result<String, Box<dyn std::error::Error>> {
        let mut git = Command::new("git");
        git.current_dir(repo_root); // コマンド実行ディレクトリを指定
        Ok(String::from_utf8(
            git.arg("rev-parse").arg("HEAD").output()?.stdout,
        )?)
    }

    /// 指定されたリポジトリルートをスキャンしてマップを生成
    pub fn generate_map(repo_root: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let id = Self::get_head_commit_id(repo_root)?;
        let result = WalkDir::new(repo_root) // 指定されたルートから探索
            .min_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|entry| {
                entry.file_type().is_file()
                    && entry.path().extension().map_or(false, |ext| ext == "txt")
            })
            .map(|entry| entry.into_path().to_str().unwrap().to_string())
            .collect();
        Ok(MapCache { id, paths: result })
    }

    /// 現在のキャッシュが指定されたリポジトリの最新コミットと一致するか確認
    pub fn is_latest(&self, repo_root: &Path) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(self.id == Self::get_head_commit_id(repo_root)?)
    }
}

/// キャッシュの保存先とリポジトリのルートを別々に指定してマップを更新
pub fn update_map(
    cache_bin_root: &Path,
    repository_root: &Path,
) -> Result<MapCache, Box<dyn std::error::Error>> {
    let cache_path: PathBuf = cache_bin_root.join("cache.bin");

    // キャッシュファイルの存在確認と読み込み
    let cache_raw: Option<Vec<u8>> = if cache_path.exists() {
        Some(fs::read(&cache_path)?)
    } else {
        None
    };

    // キャッシュの検証または新規生成
    // (mapデータ, 保存が必要かどうかのフラグ) を返す
    let (map, needs_save) = if let Some(data) = cache_raw {
        // rkyvによるゼロコピーデシリアライズの準備
        let archive_undeserialized = unsafe { archived_root::<MapCache>(&data) };
        let archive: MapCache = archive_undeserialized.deserialize(&mut rkyv::Infallible)?;

        // 最新かどうかチェック
        if !archive.is_latest(repository_root)? {
            // 古い場合は再生成し、保存フラグをtrueに
            (MapCache::generate_map(repository_root)?, true)
        } else {
            // 最新の場合はそのまま使用し、保存フラグはfalse
            (archive, false)
        }
    } else {
        // キャッシュがない場合は新規生成し、保存フラグをtrueに
        (MapCache::generate_map(repository_root)?, true)
    };

    // 変更があった場合のみディスクに書き込む
    if needs_save {
        let mut file = File::create(&cache_path)?;
        file.write_all(rkyv::to_bytes::<_, 256>(&map)?.as_slice())?;
    }

    Ok(map)
}
