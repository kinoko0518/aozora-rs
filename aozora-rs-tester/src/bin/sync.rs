use std::{
    path::{Path, PathBuf},
    process::Command,
};

use aozora_rs_tester::{REPOSITORY, update_map};

fn aozora_sync(target: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut git = Command::new("git");
    let update_aozora = if !std::fs::exists(target)? {
        git.arg("clone")
            .arg("--depth")
            .arg("1")
            .arg(format!("https://github.com/aozorahack/{}.git", REPOSITORY))
    } else {
        git.current_dir(target)
            .arg("pull")
            .arg("origin")
            .arg("master")
    };
    update_aozora.status()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let current = std::env::current_dir()?;
    let target: PathBuf = (&current).join(REPOSITORY);

    println!("青空文庫の最新版と同期中……");
    aozora_sync(&target).unwrap();

    println!("マップを更新します……");
    update_map(&current)?;

    Ok(())
}
