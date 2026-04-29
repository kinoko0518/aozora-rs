use std::fs;
use zed_extension_api::{self as zed, Result};

struct AozoraExtension;

impl zed::Extension for AozoraExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let release = zed::latest_github_release(
            "kinoko0518/aozora-rs",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (platform, arch) = zed::current_platform();
        let asset_name = match (platform, arch) {
            (zed::Os::Mac, zed::Architecture::Aarch64) => "aozora-lsp-macos-aarch64",
            (zed::Os::Mac, zed::Architecture::X8664) => "aozora-lsp-macos-aarch64",
            (zed::Os::Linux, zed::Architecture::X8664) => "aozora-lsp-linux-x86_64",
            (zed::Os::Windows, zed::Architecture::X8664) => "aozora-lsp-windows-x86_64.exe",
            _ => return Err(format!("Unsupported platform: {:?} {:?}", platform, arch)),
        };

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("no asset found matching {:?}", asset_name))?;

        let version_dir = format!("aozora-lsp-{}", release.version);
        let binary_path = format!("{}/{}", version_dir, asset_name);

        if !fs::metadata(&binary_path).is_ok_and(|stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(
                &asset.download_url,
                &binary_path,
                zed::DownloadedFileType::Uncompressed,
            )
            .map_err(|e| format!("failed to download file: {e}"))?;

            zed::make_file_executable(&binary_path)?;
        }

        Ok(zed::Command {
            command: binary_path,
            args: vec![],
            env: vec![],
        })
    }
}

zed::register_extension!(AozoraExtension);
