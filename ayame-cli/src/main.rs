use ayame_core::{
    AbstractAozoraZip, AozoraHyle, Encoding, PotentialCSS, WritingDirection,
};
use clap::{Parser, Subcommand};
use miette::{IntoDiagnostic, Result, miette};
use std::fs;
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
    /// 青空文庫书式のファイルからXHTMLを生成
    Xhtml {
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
    let mut css_contents = Vec::new();
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

fn handle_xhtml(
    source: PathBuf,
    utf8: bool,
    horizontal: bool,
    no_prelude: bool,
    no_miyabi: bool,
    extra_css: Vec<PathBuf>,
    output: Option<PathBuf>,
) -> Result<()> {
    let timer = std::time::Instant::now();
    let output_dir = get_output_dir(output)?;
    let file_stem = get_file_stem(&source)?;
    let bytes = fs::read(&source).into_diagnostic()?;

    let potential = PotentialCSS {
        use_prelude: !no_prelude,
        use_miyabi: !no_miyabi,
        direction: if horizontal {
            WritingDirection::Horizontal
        } else {
            WritingDirection::Vertical
        },
    };

    let extra_css_contents = read_extra_css(&extra_css)?;
    let extra_css_refs: Vec<&str> = extra_css_contents.iter().map(|s| s.as_str()).collect();

    let hyle = if is_zip(&source) {
        AozoraHyle::Zip((bytes, to_encoding(utf8)))
    } else {
        AozoraHyle::Txt((bytes, to_encoding(utf8)))
    };
    let abstract_zip: AbstractAozoraZip = hyle.try_into().map_err(|e| miette!("{}", e))?;

    let az_result = abstract_zip
        .generate_browser_xhtml(potential, extra_css_refs)
        .map_err(|e| miette!("{}", e))?;

    let (xhtml, errors) = az_result.into_tuple();

    for error in &errors {
        eprintln!("警告: {:?}", error);
    }

    let output_path = output_dir.join(format!("{}.xhtml", file_stem));
    fs::write(&output_path, xhtml).into_diagnostic()?;
    
    println!(
        "生成完了（{:?}）: {}",
        timer.elapsed(),
        output_path.display()
    );

    Ok(())
}

fn handle_epub(
    source: PathBuf,
    utf8: bool,
    horizontal: bool,
    no_prelude: bool,
    no_miyabi: bool,
    _extra_css: Vec<PathBuf>,
    output: Option<PathBuf>,
) -> Result<()> {
    let timer = std::time::Instant::now();
    let output_dir = get_output_dir(output)?;
    let file_stem = get_file_stem(&source)?;
    let bytes = fs::read(&source).into_diagnostic()?;

    let potential = PotentialCSS {
        use_prelude: !no_prelude,
        use_miyabi: !no_miyabi,
        direction: if horizontal {
            WritingDirection::Horizontal
        } else {
            WritingDirection::Vertical
        },
    };

    let hyle = if is_zip(&source) {
        AozoraHyle::Zip((bytes, to_encoding(utf8)))
    } else {
        AozoraHyle::Txt((bytes, to_encoding(utf8)))
    };
    let abstract_zip: AbstractAozoraZip = hyle.try_into().map_err(|e| miette!("{}", e))?;

    let output_path = output_dir.join(format!("{}.epub", file_stem));
    let mut file = fs::File::create(&output_path).into_diagnostic()?;

    let az_result = abstract_zip
        .generate_epub(&mut file, potential, "ja")
        .map_err(|e| miette!("{}", e))?;

    let ((), errors) = az_result.into_tuple();

    for error in &errors {
        eprintln!("警告: {:?}", error);
    }

    println!(
        "生成完了（{:?}）: {}",
        timer.elapsed(),
        output_path.display()
    );

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Xhtml {
            source,
            utf8,
            horizontal,
            no_prelude,
            no_miyabi,
            css,
            output,
        } => {
            handle_xhtml(
                source, utf8, horizontal, no_prelude, no_miyabi, css, output,
            )?;
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
