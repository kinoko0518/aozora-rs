use rkyv::{Archive, Deserialize, Serialize, access_unchecked, rancor::Error};
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};
use walkdir::WalkDir;

use crate::REPOSITORY;

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[repr(C)]
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
        Ok(MapCache {
            id: id,
            paths: result,
        })
    }

    pub fn is_latest(&self) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(self.id == Self::get_head_commit_id()?)
    }
}

pub fn update_map(base_path: &Path) -> Result<MapCache, Box<dyn std::error::Error>> {
    let cache_path: PathBuf = (&base_path).join("cache.bin");

    println!("既存マップを確認中……");
    let cache_raw: Option<Vec<u8>> = if std::fs::exists(&cache_path)? {
        Some(std::fs::read(&cache_path)?)
    } else {
        None
    };
    let map = if let Some(data) = cache_raw {
        println!("既存マップが存在します。最新かを確認します……");
        let archive_undeserialized = unsafe { access_unchecked::<ArchivedMapCache>(&data) };
        let archive = rkyv::deserialize::<MapCache, rkyv::rancor::Error>(archive_undeserialized)?;
        if !archive.is_latest()? {
            println!("マップが古くなっています。更新します……");
            MapCache::generate_map()?
        } else {
            println!("マップは最新です。更新プロセスをスキップします……");
            archive
        }
    } else {
        println!("マップが存在しません。作成します……");
        MapCache::generate_map()?
    };

    let mut file = File::create(&cache_path)?;
    file.write_all(rkyv::to_bytes::<Error>(&map)?.as_slice())?;

    println!("マップの更新が完了しました。");
    Ok(map)
}
