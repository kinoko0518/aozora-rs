use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::REPOSITORY;

/// Progress states for git sync operations
#[derive(Debug, Clone)]
pub enum GitSyncProgress {
    /// Checking if repository exists
    Checking,
    /// Cloning repository (doesn't exist yet)
    Cloning,
    /// Pulling latest changes
    Pulling,
    /// Sync completed successfully
    Done,
    /// Error occurred
    Error(String),
}

/// Synchronize the Aozora Bunko repository.
///
/// This is a pure function that reports progress via callback.
pub fn sync_repository<F>(
    base_path: &Path,
    mut on_progress: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnMut(GitSyncProgress),
{
    let target: PathBuf = base_path.join(REPOSITORY);

    on_progress(GitSyncProgress::Checking);

    let mut git = Command::new("git");
    let update_aozora = if !std::fs::exists(&target)? {
        on_progress(GitSyncProgress::Cloning);
        git.arg("clone")
            .arg("--depth")
            .arg("1")
            .arg(format!("https://github.com/aozorahack/{}.git", REPOSITORY))
            .current_dir(base_path)
    } else {
        on_progress(GitSyncProgress::Pulling);
        git.current_dir(&target)
            .arg("pull")
            .arg("origin")
            .arg("master")
    };

    match update_aozora.output() {
        Ok(output) if output.status.success() => {
            on_progress(GitSyncProgress::Done);
            Ok(())
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let err = format!("git failed: {}", stderr.trim());
            on_progress(GitSyncProgress::Error(err.clone()));
            Err(err.into())
        }
        Err(e) => {
            let err = format!("Failed to run git: {}", e);
            on_progress(GitSyncProgress::Error(err.clone()));
            Err(err.into())
        }
    }
}

/// Synchronize without progress callback (for simple CLI usage)
pub fn sync_repository_simple(base_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    sync_repository(base_path, |_| {})
}
