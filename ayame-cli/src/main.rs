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

        /// 入力ファイルがUTF-8でエンコードされている場合に指定（デフォルト: Shift-JIS）
        #[arg(long)]
        utf8: bool,

        /// 出力先ディレクトリ（デフォルト: カレントディレクトリ）
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// 青空文庫書式のファイルからEPUBを生成
    Epub {
        /// 入力ファイル（.txt または .zip）
        source: PathBuf,

        /// 入力ファイルがUTF-8でエンコードされている場合に指定（デフォルト: Shift-JIS）
        #[arg(long)]
        utf8: bool,

        /// 横書きで生成（デフォルト: 縦書き）
        #[arg(long)]
        horizontal: bool,

        /// preludeを適用しない
        #[arg(long)]
        no_prelude: bool,

        /// miyabiを適用しない
        #[arg(long)]
        no_miyabi: bool,

        /// 追加で適用するCSSファイルのパス（複数指定可）
        #[arg(long)]
        css: Vec<PathBuf>,

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

fn to_encoding(utf8: bool) -> Encoding {
    if utf8 {
        Encoding::Utf8
    } else {
        Encoding::ShiftJis
    }
}

fn is_zip(source: &Path) -> bool {
    source
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("zip"))
        .unwrap_or(false)
}

/// CSSコンテンツリストを構築する
fn resolve_css(
    direction: &WritingDirection,
    no_prelude: bool,
    no_miyabi: bool,
    extra_css: &[PathBuf],
) -> Result<Vec<String>> {
    let mut css_contents = vec![layout_css(direction).to_string()];

    if !no_prelude {
        css_contents.push(resolve_builtin_css("prelude").unwrap().to_string());
    }
    if !no_miyabi {
        css_contents.push(resolve_builtin_css("miyabi").unwrap().to_string());
    }

    for path in extra_css {
        if path.exists() {
            let content = fs::read_to_string(path).into_diagnostic()?;
            css_contents.push(content);
        } else {
            return Err(miette!("CSSファイルが見つかりません: {}", path.display()));
        }
    }

    Ok(css_contents)
}

fn handle_xhtml(source: PathBuf, merge: bool, utf8: bool, output: Option<PathBuf>) -> Result<()> {
    let output_dir = get_output_dir(output)?;
    let file_stem = get_file_stem(&source)?;
    let bytes = fs::read(&source).into_diagnostic()?;

    let (xhtmls, errors) = generate_xhtml(&bytes, is_zip(&source), &to_encoding(utf8))
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
    utf8: bool,
    horizontal: bool,
    no_prelude: bool,
    no_miyabi: bool,
    extra_css: Vec<PathBuf>,
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

    let css_contents = resolve_css(&direction, no_prelude, no_miyabi, &extra_css)?;
    let css_refs: Vec<&str> = css_contents.iter().map(|s| s.as_str()).collect();

    let setting = EpubSetting {
        language: "ja",
        is_rtl: !horizontal,
    };

    let epub_bytes = generate_epub(
        &bytes,
        is_zip(&source),
        &to_encoding(utf8),
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
            utf8,
            output,
        } => {
            handle_xhtml(source, merge, utf8, output)?;
        }
        Commands::Epub {
            source,
            utf8,
            horizontal,
            no_prelude,
            no_miyabi,
            css,
            output,
        } => {
            handle_epub(source, utf8, horizontal, no_prelude, no_miyabi, css, output)?;
        }
    }

    Ok(())
}
