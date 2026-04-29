use rkyv::{Archive, Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    process::Command,
};
use walkdir::WalkDir;

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[rkyv(derive(Debug))]
pub struct MapCache {
    id: String,
    pub paths: Vec<String>,
}

impl MapCache {
    /// 指定されたリポジトリルートで git rev-parse HEAD を実行してコミットIDを取得
    fn get_head_commit_id(repo_root: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut git = Command::new("git");
        git.current_dir(repo_root); // コマンド実行ディレクトリを指定
        Ok(String::from_utf8(
            git.arg("rev-parse").arg("HEAD").output()?.stdout,
        )?)
    }

    /// 指定されたリポジトリルートをスキャンしてマップを生成
    pub fn generate_map(repo_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let id = Self::get_head_commit_id(repo_path)?;
        let result = WalkDir::new(repo_path) // 指定されたルートから探索
            .min_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|entry| {
                entry.file_type().is_file()
                    && entry.path().extension().is_some_and(|ext| ext == "txt")
            })
            .map(|entry| entry.into_path().to_string_lossy().to_string())
            .collect();
        Ok(MapCache { id, paths: result })
    }

    /// 現在のキャッシュが指定されたリポジトリの最新コミットと一致するか確認
    pub fn is_latest(&self, repo_root: &str) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(self.id == Self::get_head_commit_id(repo_root)?)
    }
}

/// キャッシュの保存先とリポジトリのルートを別々に指定してマップを更新
pub fn update_map(
    cache_bin: &str,
    repo_path: &str,
) -> Result<MapCache, Box<dyn std::error::Error>> {
    // キャッシュファイルの存在確認と読み込み
    let cache_raw: Option<Vec<u8>> = fs::read(cache_bin).ok();

    // キャッシュの検証または新規生成
    // (mapデータ, 保存が必要かどうかのフラグ) を返す
    let (map, needs_save) = if let Some(data) = cache_raw {
        let archive = rkyv::from_bytes::<MapCache, rkyv::rancor::Error>(&data)?;

        // 最新かどうかチェック
        if !archive.is_latest(repo_path)? {
            // 古い場合は再生成し、保存フラグをtrueに
            (MapCache::generate_map(repo_path)?, true)
        } else {
            // 最新の場合はそのまま使用し、保存フラグはfalse
            (archive, false)
        }
    } else {
        // キャッシュがない場合は新規生成し、保存フラグをtrueに
        (MapCache::generate_map(repo_path)?, true)
    };

    // 変更があった場合のみディスクに書き込む
    if needs_save {
        let mut file = File::create(cache_bin)?;
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&map)?;
        file.write_all(&bytes)?;
    }

    Ok(map)
}
