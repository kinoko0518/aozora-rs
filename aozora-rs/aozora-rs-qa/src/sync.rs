use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::REPOSITORY;

pub fn sync_repository(base_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let target: PathBuf = base_path.join(REPOSITORY);

    let mut git = Command::new("git");
    let update_aozora = if !std::fs::exists(&target)? {
        git.arg("clone")
            .arg("--depth")
            .arg("1")
            .arg(format!("https://github.com/aozorahack/{}.git", REPOSITORY))
            .current_dir(base_path)
    } else {
        git.current_dir(&target)
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
