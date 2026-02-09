use aozora_rs::{XHTMLResult, convert_with_meta};
use clap::{Parser, Subcommand};
use encoding_rs::SHIFT_JIS;
use miette::{IntoDiagnostic, Result, miette};
use std::fs;
use std::io::{Cursor, Write};
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

/// Shift-JISバイト列をUTF-8文字列に変換し、\r\nを\nに置換
fn decode_shift_jis(bytes: &[u8]) -> Result<String> {
    let (decoded, _, had_errors) = SHIFT_JIS.decode(bytes);
    if had_errors {
        return Err(miette!("Shift-JISデコード中にエラーが発生しました"));
    }
    // Replace \r\n with \n
    Ok(decoded.replace("\r\n", "\n"))
}

/// バイト列をUTF-8文字列に変換（sjisフラグに応じて処理を分岐）
fn decode_bytes(bytes: &[u8], sjis: bool) -> Result<String> {
    if sjis {
        decode_shift_jis(bytes)
    } else {
        String::from_utf8(bytes.to_vec())
            .map_err(|_| miette!("UTF-8として読み取れませんでした。Shift-JISファイルの場合は --sjis オプションを指定してください"))
    }
}

fn read_source_text(source: &Path, sjis: bool) -> Result<String> {
    let ext = source.extension().and_then(|e| e.to_str()).unwrap_or("");

    match ext.to_lowercase().as_str() {
        "txt" => {
            let bytes = fs::read(source).into_diagnostic()?;
            decode_bytes(&bytes, sjis)
        }
        "zip" => {
            let bytes = fs::read(source).into_diagnostic()?;
            let azz = aozora_rs::AozoraZip::read_from_zip_with_encoding(&bytes, sjis)
                .map_err(|e| miette!("{}", e))?;
            Ok(azz.text)
        }
        _ => Err(miette!("サポートされていないファイル形式です: .{}", ext)),
    }
}

fn write_xhtml_files(
    xhtmls: &XHTMLResult,
    output_dir: &Path,
    file_stem: &str,
    merge: bool,
) -> Result<()> {
    if merge {
        // Merge all XHTMLs with <hr> delimiter
        let merged = xhtmls.xhtmls.join("\n<hr />\n");
        let output_path = output_dir.join(format!("{}.xhtml", file_stem));
        let mut file = fs::File::create(&output_path).into_diagnostic()?;
        file.write_all(merged.as_bytes()).into_diagnostic()?;
        println!("生成完了: {}", output_path.display());
    } else {
        // Write individual XHTML files
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

fn handle_xhtml(source: PathBuf, merge: bool, sjis: bool, output: Option<PathBuf>) -> Result<()> {
    let output_dir = get_output_dir(output)?;
    let file_stem = get_file_stem(&source)?;
    let text = read_source_text(&source, sjis)?;

    let result = convert_with_meta(&text);

    // Print errors if any
    for error in &result.errors {
        eprintln!("警告: {:?}", error);
    }

    write_xhtml_files(&result.xhtmls, &output_dir, &file_stem, merge)?;

    Ok(())
}

fn handle_epub(source: PathBuf, sjis: bool, output: Option<PathBuf>) -> Result<()> {
    let output_dir = get_output_dir(output)?;
    let file_stem = get_file_stem(&source)?;

    let ext = source.extension().and_then(|e| e.to_str()).unwrap_or("");

    let bytes = fs::read(&source).into_diagnostic()?;

    let azz = match ext.to_lowercase().as_str() {
        "zip" => aozora_rs::AozoraZip::read_from_zip_with_encoding(&bytes, sjis)
            .map_err(|e| miette!("{}", e))?,
        "txt" => {
            let text = decode_bytes(&bytes, sjis)?;
            aozora_rs::AozoraZip {
                text,
                images: std::collections::HashMap::new(),
                css: std::collections::HashMap::new(),
            }
        }
        _ => return Err(miette!("サポートされていないファイル形式です: .{}", ext)),
    };

    let output_path = output_dir.join(format!("{}.epub", file_stem));
    let mut epub_buffer = Cursor::new(Vec::new());

    let result = aozora_rs::from_aozora_zip::<Cursor<Vec<u8>>>(&mut epub_buffer, azz, Vec::new())
        .map_err(|e| miette!("{}", e))?;

    // Print any warnings using into_tuple() to access private errors
    let (_, errors) = result.into_tuple();
    for error in &errors {
        eprintln!("警告: {:?}", error);
    }

    fs::write(&output_path, epub_buffer.into_inner()).into_diagnostic()?;
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
            output,
        } => {
            handle_epub(source, sjis, output)?;
        }
    }

    Ok(())
}
