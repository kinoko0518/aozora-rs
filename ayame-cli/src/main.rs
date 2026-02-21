use aozora_rs::EpubSetting;
use ayame_core::{
    Encoding, WritingDirection, generate_epub, generate_xhtml, layout_css, resolve_builtin_css,
};
use clap::{Parser, Subcommand};
use miette::{IntoDiagnostic, Result, miette};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "ayame")]
#[command(author, version, about = "青空文庫書式からXHTML/EPUBを生成するCLIツール", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 青空文庫書式のファイルからXHTMLを生成
    Xhtml {
        /// 入力ファイル（.txt または .zip）
        source: PathBuf,

        /// 複数XHTMLを<hr>区切りで1つに結合
        #[arg(long)]
        merge: bool,

        /// 入力ファイルがShift-JISでエンコードされている場合に指定
        #[arg(long)]
        sjis: bool,

        /// 出力先ディレクトリ（デフォルト: カレントディレクトリ）
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// 青空文庫書式のファイルからEPUBを生成
    Epub {
        /// 入力ファイル（.txt または .zip）
        source: PathBuf,

        /// 入力ファイルがShift-JISでエンコードされている場合に指定
        #[arg(long)]
        sjis: bool,

        /// 横書きで生成（デフォルト: 縦書き）
        #[arg(long)]
        horizontal: bool,

        /// 適用するCSS（組み込み名またはファイルパス、複数指定可）
        #[arg(long)]
        css: Vec<String>,

        /// 出力先ディレクトリ（デフォルト: カレントディレクトリ）
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn get_output_dir(output: Option<PathBuf>) -> Result<PathBuf> {
    match output {
        Some(path) => {
            if !path.exists() {
                fs::create_dir_all(&path).into_diagnostic()?;
            }
            Ok(path)
        }
        None => std::env::current_dir().into_diagnostic(),
    }
}

fn get_file_stem(source: &Path) -> Result<String> {
    source
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .ok_or_else(|| miette!("ファイル名を取得できませんでした"))
}

fn to_encoding(sjis: bool) -> Encoding {
    if sjis {
        Encoding::ShiftJis
    } else {
        Encoding::Utf8
    }
}

fn is_zip(source: &Path) -> bool {
    source
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("zip"))
        .unwrap_or(false)
}

/// CSS名リストからCSS文字列リストを構築する
fn resolve_css(css_names: &[String], direction: &WritingDirection) -> Result<Vec<String>> {
    let mut css_contents = vec![layout_css(direction).to_string()];

    for name in css_names {
        if let Some(builtin) = resolve_builtin_css(name) {
            css_contents.push(builtin.to_string());
        } else {
            let path = Path::new(name);
            if path.exists() {
                let content = fs::read_to_string(path).into_diagnostic()?;
                css_contents.push(content);
            } else {
                return Err(miette!("CSSファイルが見つかりません: {}", name));
            }
        }
    }

    Ok(css_contents)
}

fn handle_xhtml(source: PathBuf, merge: bool, sjis: bool, output: Option<PathBuf>) -> Result<()> {
    let output_dir = get_output_dir(output)?;
    let file_stem = get_file_stem(&source)?;
    let bytes = fs::read(&source).into_diagnostic()?;

    let (xhtmls, errors) = generate_xhtml(&bytes, is_zip(&source), &to_encoding(sjis))
        .map_err(|e| miette!("{}", e))?;

    for error in &errors {
        eprintln!("警告: {:?}", error);
    }

    if merge {
        let merged = xhtmls.xhtmls.join("\n<hr />\n");
        let output_path = output_dir.join(format!("{}.xhtml", file_stem));
        let mut file = fs::File::create(&output_path).into_diagnostic()?;
        file.write_all(merged.as_bytes()).into_diagnostic()?;
        println!("生成完了: {}", output_path.display());
    } else {
        for (i, xhtml) in xhtmls.xhtmls.iter().enumerate() {
            let filename = if xhtmls.xhtmls.len() == 1 {
                format!("{}.xhtml", file_stem)
            } else {
                format!("{}_{:03}.xhtml", file_stem, i + 1)
            };
            let output_path = output_dir.join(&filename);
            let mut file = fs::File::create(&output_path).into_diagnostic()?;
            file.write_all(xhtml.as_bytes()).into_diagnostic()?;
            println!("生成完了: {}", output_path.display());
        }
    }

    Ok(())
}

fn handle_epub(
    source: PathBuf,
    sjis: bool,
    horizontal: bool,
    css_names: Vec<String>,
    output: Option<PathBuf>,
) -> Result<()> {
    let output_dir = get_output_dir(output)?;
    let file_stem = get_file_stem(&source)?;
    let bytes = fs::read(&source).into_diagnostic()?;

    let direction = if horizontal {
        WritingDirection::Horizontal
    } else {
        WritingDirection::Vertical
    };

    let css_contents = resolve_css(&css_names, &direction)?;
    let css_refs: Vec<&str> = css_contents.iter().map(|s| s.as_str()).collect();

    let setting = EpubSetting {
        language: "ja",
        is_rtl: !horizontal,
    };

    let epub_bytes = generate_epub(
        &bytes,
        is_zip(&source),
        &to_encoding(sjis),
        css_refs,
        setting,
    )
    .map_err(|e| miette!("{}", e))?;

    let output_path = output_dir.join(format!("{}.epub", file_stem));
    fs::write(&output_path, epub_bytes).into_diagnostic()?;
    println!("生成完了: {}", output_path.display());

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Xhtml {
            source,
            merge,
            sjis,
            output,
        } => {
            handle_xhtml(source, merge, sjis, output)?;
        }
        Commands::Epub {
            source,
            sjis,
            horizontal,
            css,
            output,
        } => {
            handle_epub(source, sjis, horizontal, css, output)?;
        }
    }

    Ok(())
}
