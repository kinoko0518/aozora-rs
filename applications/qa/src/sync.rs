use std::path::Path;
use std::process::Command;

use crate::REPOSITORY;

pub fn sync_repository(repo_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let repo = Path::new(repo_path);
    let mut git = Command::new("git");
    let update_aozora = if !repo.exists() {
        let parent = repo
            .parent()
            .ok_or("リポジトリパスの親ディレクトリが取得できません")?;
        std::fs::create_dir_all(parent)?;
        git.arg("clone")
            .arg("--depth")
            .arg("1")
            .arg(format!("https://github.com/aozorahack/{}.git", REPOSITORY))
            .arg(repo_path)
            .current_dir(parent)
    } else {
        git.current_dir(repo_path)
            .arg("pull")
            .arg("origin")
            .arg("master")
    };

    match update_aozora.output() {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let err = format!("git failed: {}", stderr.trim());
            Err(err.into())
        }
        Err(e) => {
            let err = format!("Failed to run git: {}", e);
            Err(err.into())
        }
    }
}
