use std::io::Cursor;

use ayame::{
    AozoraDocument, AozoraZip, Dependencies, Encoding, PageInjectors, Style, WritingDirection,
};
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
    fn to_style<'a>(&self, extra_css: &'a [String]) -> Style<'a> {
        let mut style = Style::default();
        style
            .prelude(!self.no_prelude)
            .direction(if self.horizontal {
                WritingDirection::Horizontal
            } else {
                WritingDirection::Vertical
            });
        if !self.no_miyabi {
            ayame::apply_miyabi(&mut style);
        }
        for css in extra_css {
            style.add_css(css.as_str());
        }
        style
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

/// ソースファイルを読み込み、テキストと画像依存を返す
fn read_source(source: &Path, encoding: &Encoding, gaiji: bool) -> Result<(String, Dependencies)> {
    let bytes = fs::read(source).into_diagnostic()?;
    let (text, deps) = if is_zip(source) {
        let azz =
            AozoraZip::read_from_zip(Cursor::new(bytes), encoding).map_err(|e| miette!("{}", e))?;
        (azz.txt, azz.images)
    } else {
        let txt = encoding
            .bytes_to_string(bytes)
            .map_err(|e| miette!("{}", e))?;
        (txt, Dependencies::default())
    };
    let text = if gaiji {
        aozora_rs::utf8tify_all_gaiji(&text).into_owned()
    } else {
        text
    };
    Ok((text, deps))
}

fn handle_xhtml(source: &Path, args: &CommonArgs, style: &Style, output_dir: &Path) -> Result<()> {
    let timer = std::time::Instant::now();
    let file_stem = get_file_stem(source)?;

    let (text, deps) = read_source(source, &to_encoding(args.utf8), !args.no_gaiji)?;
    let doc = AozoraDocument::from_str(&text, Some(&deps)).map_err(|e| miette!("{}", e))?;

    let (xhtml, errors) = ayame::to_browser_xhtml(&doc, style).map_err(|e| miette!("{}", e))?;
    for error in &errors {
        eprintln!("警告 ({}): {}", source.display(), error.display(&text));
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

fn handle_epub(source: &Path, args: &CommonArgs, style: &Style, output_dir: &Path) -> Result<()> {
    let timer = std::time::Instant::now();

    let (text, deps) = read_source(source, &to_encoding(args.utf8), !args.no_gaiji)?;
    let doc = AozoraDocument::from_str(&text, Some(&deps)).map_err(|e| miette!("{}", e))?;

    let output_path = output_dir.join(format!("[{}] {}.epub", doc.meta.author, doc.meta.title));
    let mut file = fs::File::create(&output_path).into_diagnostic()?;

    let injectors = PageInjectors {
        title_page: Some(ayame::title_page_writer()),
        toc_page: Some(ayame::toc_page_writer()),
    };

    let warnings = doc
        .epub(&mut file, style, &injectors)
        .map_err(|e| miette!("{}", e))?;
    for w in &warnings {
        eprintln!("警告 ({}): {}", source.display(), w.display(&text));
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
            let extra_css_contents = read_extra_css(&args.css)?;
            let style = args.to_style(&extra_css_contents);

            args.sources.par_iter().for_each(|source| {
                if let Err(e) = handle_xhtml(source, args, &style, &output_dir) {
                    eprintln!("エラー ({}): {:?}", source.display(), e);
                }
            });
        }
        Commands::Epub(args) => {
            let output_dir = get_output_dir(&args.output)?;
            let extra_css_contents = read_extra_css(&args.css)?;
            let style = args.to_style(&extra_css_contents);

            args.sources.par_iter().for_each(|source| {
                if let Err(e) = handle_epub(source, args, &style, &output_dir) {
                    eprintln!("エラー ({}): {:?}", source.display(), e);
                }
            });
        }
    }

    Ok(())
}
