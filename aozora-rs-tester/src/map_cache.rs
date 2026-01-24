use rkyv::{Archive, Deserialize, Serialize, archived_root};
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};
use walkdir::WalkDir;

use crate::REPOSITORY;

/// Progress states for map cache operations
#[derive(Debug, Clone)]
pub enum MapCacheProgress {
    /// Checking if cache exists
    CheckingCache,
    /// Cache file found
    CacheFound,
    /// Cache is outdated, will regenerate
    CacheOutdated,
    /// Cache is up to date
    CacheUpToDate,
    /// Cache file not found, will create
    CacheNotFound,
    /// Generating new map from files
    GeneratingMap,
    /// Saving cache to disk
    SavingCache,
    /// Operation completed
    Done,
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[archive_attr(derive(Debug))]
pub struct MapCache {
    id: String,
    pub paths: Vec<String>,
}

impl MapCache {
    fn get_head_commit_id() -> Result<String, Box<dyn std::error::Error>> {
        let mut git = Command::new("git");
        Ok(String::from_utf8(
            git.arg("rev-parse").arg("HEAD").output()?.stdout,
        )?)
    }

    pub fn generate_map() -> Result<Self, Box<dyn std::error::Error>> {
        let id = Self::get_head_commit_id()?;
        let result = WalkDir::new(REPOSITORY)
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

    pub fn is_latest(&self) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(self.id == Self::get_head_commit_id()?)
    }
}

/// Update map cache with progress callback.
///
/// This is a pure function - all progress reporting is done via callback.
pub fn update_map_with_progress<F>(
    base_path: &Path,
    mut on_progress: F,
) -> Result<MapCache, Box<dyn std::error::Error>>
where
    F: FnMut(MapCacheProgress),
{
    let cache_path: PathBuf = base_path.join("cache.bin");

    on_progress(MapCacheProgress::CheckingCache);

    let cache_raw: Option<Vec<u8>> = if std::fs::exists(&cache_path)? {
        Some(std::fs::read(&cache_path)?)
    } else {
        None
    };

    let map = if let Some(data) = cache_raw {
        on_progress(MapCacheProgress::CacheFound);
        let archive_undeserialized = unsafe { archived_root::<MapCache>(&data) };
        let archive: MapCache = archive_undeserialized.deserialize(&mut rkyv::Infallible)?;
        if !archive.is_latest()? {
            on_progress(MapCacheProgress::CacheOutdated);
            on_progress(MapCacheProgress::GeneratingMap);
            MapCache::generate_map()?
        } else {
            on_progress(MapCacheProgress::CacheUpToDate);
            archive
        }
    } else {
        on_progress(MapCacheProgress::CacheNotFound);
        on_progress(MapCacheProgress::GeneratingMap);
        MapCache::generate_map()?
    };

    on_progress(MapCacheProgress::SavingCache);
    let mut file = File::create(&cache_path)?;
    file.write_all(rkyv::to_bytes::<_, 256>(&map)?.as_slice())?;

    on_progress(MapCacheProgress::Done);
    Ok(map)
}

/// Update map cache without progress callback (backward compatible).
pub fn update_map(base_path: &Path) -> Result<MapCache, Box<dyn std::error::Error>> {
    update_map_with_progress(base_path, |_| {})
}
