use rkyv::{Archive, Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
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
    /// 指定されたパスの識別子を取得（GitコミットID、または更新時刻のフォールバック）
    fn get_path_id(target_path: &Path) -> String {
        if target_path.is_dir() {
            let mut git = Command::new("git");
            git.current_dir(target_path);
            if let Ok(output) = git.arg("rev-parse").arg("HEAD").output()
                && output.status.success()
                && let Ok(s) = String::from_utf8(output.stdout)
            {
                return s.trim().to_string();
            }
        }

        // Gitリポジトリでない場合、または単一ファイルの場合は更新時刻から識別子を生成
        target_path
            .metadata()
            .and_then(|m| m.modified())
            .map(|t| format!("{:?}", t))
            .unwrap_or_else(|_| "unknown".to_string())
    }

    /// 指定されたパスを走査してマップを生成（単一ファイル、非Gitディレクトリにも対応）
    pub fn generate_map(target_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let path = Path::new(target_path);

        if path.is_file() {
            if path.extension().is_some_and(|ext| ext == "txt") {
                return Ok(MapCache {
                    id: Self::get_path_id(path),
                    paths: vec![path.to_string_lossy().to_string()],
                });
            } else {
                return Ok(MapCache {
                    id: Self::get_path_id(path),
                    paths: Vec::new(),
                });
            }
        }

        let id = Self::get_path_id(path);
        let mut result: Vec<String> = WalkDir::new(path)
            .min_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|entry| {
                entry.file_type().is_file()
                    && entry.path().extension().is_some_and(|ext| ext == "txt")
            })
            .map(|entry| entry.into_path().to_string_lossy().to_string())
            .collect();

        result.sort();

        Ok(MapCache { id, paths: result })
    }

    /// 現在のキャッシュが指定されたパスの状態と一致するか確認
    pub fn is_latest(&self, target_path: &str) -> bool {
        let path = Path::new(target_path);
        self.id == Self::get_path_id(path)
    }
}

/// キャッシュの保存先と対象パスを指定してマップを更新
pub fn update_map(
    cache_bin: &str,
    target_path: &str,
) -> Result<MapCache, Box<dyn std::error::Error>> {
    let cache_raw: Option<Vec<u8>> = fs::read(cache_bin).ok();

    let (map, needs_save) = if let Some(data) = cache_raw {
        if let Ok(archive) = rkyv::from_bytes::<MapCache, rkyv::rancor::Error>(&data) {
            if !archive.is_latest(target_path) {
                (MapCache::generate_map(target_path)?, true)
            } else {
                (archive, false)
            }
        } else {
            (MapCache::generate_map(target_path)?, true)
        }
    } else {
        (MapCache::generate_map(target_path)?, true)
    };

    if needs_save {
        if let Some(parent) = Path::new(cache_bin).parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = File::create(cache_bin)?;
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&map)?;
        file.write_all(&bytes)?;
    }

    Ok(map)
}
