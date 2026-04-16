use ayame::{AbstractAozoraZip, AozoraHyle, Encoding, PotentialCSS, WritingDirection};
use clap::{Args, Parser, Subcommand};
use miette::{IntoDiagnostic, Result, miette};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "ayame")]
#[command(
    author,
    version,
    about = "青空文庫書式からXHTML/EPUBを生成するCLIツール"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 青空文庫書式のファイルからXHTMLを生成
    Xhtml(CommonArgs),
    /// 青空文庫書式のファイルからEPUBを生成
    Epub(CommonArgs),
}

/// XhtmlとEpubで共通のコマンドライン引数
#[derive(Args)]
struct CommonArgs {
    #[arg(required = true)]
    sources: Vec<PathBuf>,

    #[arg(long)]
    utf8: bool,

    #[arg(long)]
    horizontal: bool,

    #[arg(long)]
    no_prelude: bool,

    #[arg(long)]
    no_miyabi: bool,

    #[arg(long)]
    css: Vec<PathBuf>,

    #[arg(long)]
    no_gaiji: bool,

    #[arg(short, long)]
    output: Option<PathBuf>,
}

impl CommonArgs {
    /// 引数からPotentialCSSを生成する
    fn to_potential_css(&self) -> PotentialCSS {
        PotentialCSS {
            use_prelude: !self.no_prelude,
            use_miyabi: !self.no_miyabi,
            direction: if self.horizontal {
                WritingDirection::Horizontal
            } else {
                WritingDirection::Vertical
            },
        }
    }
}

fn get_output_dir(output: &Option<PathBuf>) -> Result<PathBuf> {
    match output {
        Some(path) => {
            if !path.exists() {
                fs::create_dir_all(path).into_diagnostic()?;
            }
            Ok(path.clone())
        }
        None => std::env::current_dir().into_diagnostic(),
    }
}

fn get_file_stem(source: &Path) -> Result<String> {
    source
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .ok_or_else(|| miette!("ファイル名を取得できませんでした: {}", source.display()))
}

fn to_encoding(utf8: bool) -> Encoding {
    if utf8 {
        Encoding::Utf8
    } else {
        Encoding::ShiftJIS
    }
}

fn is_zip(source: &Path) -> bool {
    source
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("zip"))
        .unwrap_or(false)
}

fn read_extra_css(extra_css: &[PathBuf]) -> Result<Vec<String>> {
    extra_css
        .iter()
        .map(|path| {
            if path.exists() {
                fs::read_to_string(path).into_diagnostic()
            } else {
                Err(miette!("CSSファイルが見つかりません: {}", path.display()))
            }
        })
        .collect()
}

// --- 処理コア ---

/// ファイルを読み込み、AozoraHyleを構築する共通処理
fn read_hyle(source: &Path, utf8: bool) -> Result<AozoraHyle> {
    let bytes = fs::read(source).into_diagnostic()?;

    if is_zip(source) {
        Ok(AozoraHyle::Zip((bytes, to_encoding(utf8))))
    } else {
        Ok(AozoraHyle::Txt((bytes, to_encoding(utf8))))
    }
}

fn handle_xhtml(
    source: &Path,
    args: &CommonArgs,
    potential: &PotentialCSS,
    extra_css_refs: &[&str],
    output_dir: &Path,
) -> Result<()> {
    let timer = std::time::Instant::now();
    let file_stem = get_file_stem(source)?;

    let hyle = read_hyle(source, args.utf8)?;
    let (string, dependencies) = hyle.encode(!args.no_gaiji).map_err(|e| miette!("{}", e))?;
    let abstract_zip = AbstractAozoraZip::from_str_with_meta(string.as_str(), dependencies)
        .map_err(|e| miette!("{}", e))?;

    let az_result = abstract_zip
        .browser_xhtml(potential, extra_css_refs.to_vec())
        .map_err(|e| miette!("{}", e))?;

    let (xhtml, errors) = az_result.into_tuple();
    for error in &errors {
        eprintln!("警告 ({}): {:?}", source.display(), error);
    }

    let output_path = output_dir.join(format!("{}.xhtml", file_stem));
    fs::write(&output_path, xhtml).into_diagnostic()?;

    println!(
        "生成完了 [{:?}] -> {}",
        timer.elapsed(),
        output_path.display()
    );
    Ok(())
}

fn handle_epub(
    source: &Path,
    args: &CommonArgs,
    potential: &PotentialCSS,
    output_dir: &Path,
) -> Result<()> {
    let timer = std::time::Instant::now();

    let hyle = read_hyle(source, args.utf8)?;
    let (string, dependencies) = hyle.encode(!args.no_gaiji).map_err(|e| miette!("{}", e))?;
    let abstract_zip = AbstractAozoraZip::from_str_with_meta(string.as_str(), dependencies)
        .map_err(|e| miette!("{}", e))?;

    let meta = abstract_zip
        .meta
        .as_ref()
        .ok_or_else(|| miette!("メタデータが取得できません"))?;

    let output_path = output_dir.join(format!("[{}] {}.epub", meta.author, meta.title));
    let mut file = fs::File::create(&output_path).into_diagnostic()?;

    let az_result = abstract_zip
        .epub(&mut file, potential, "ja")
        .map_err(|e| miette!("{}", e))?;

    let ((), errors) = az_result.into_tuple();
    for error in &errors {
        eprintln!("警告 ({}): {:?}", source.display(), error);
    }

    println!(
        "生成完了 [{:?}] -> {}",
        timer.elapsed(),
        output_path.display()
    );
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Xhtml(args) => {
            let output_dir = get_output_dir(&args.output)?;
            let potential = args.to_potential_css();
            let extra_css_contents = read_extra_css(&args.css)?;
            let extra_css_refs: Vec<&str> = extra_css_contents.iter().map(|s| s.as_str()).collect();

            args.sources.par_iter().for_each(|source| {
                if let Err(e) = handle_xhtml(source, args, &potential, &extra_css_refs, &output_dir)
                {
                    eprintln!("エラー ({}): {:?}", source.display(), e);
                }
            });
        }
        Commands::Epub(args) => {
            let output_dir = get_output_dir(&args.output)?;
            let potential = args.to_potential_css();

            args.sources.par_iter().for_each(|source| {
                if let Err(e) = handle_epub(source, args, &potential, &output_dir) {
                    eprintln!("エラー ({}): {:?}", source.display(), e);
                }
            });
        }
    }

    Ok(())
}
