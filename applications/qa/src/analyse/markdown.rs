use std::fs::File;
use std::io::Write;
use std::path::Path;

use super::metrics::PerformanceReport;

pub fn write_markdown_report(
    report: &PerformanceReport,
    out_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(out_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut f = File::create(path)?;

    writeln!(f, "# aozora-rs パフォーマンス解析レポート\n")?;

    writeln!(f, "## 概要\n")?;
    writeln!(f, "| 項目 | 値 |")?;
    writeln!(f, "|:---|---:|")?;
    writeln!(f, "| 対象作品数 | {} 作品 |", report.total_works)?;
    writeln!(f, "| 成功作品数 | {} 作品 |", report.total_succeed)?;
    writeln!(f, "| 失敗作品数 | {} 作品 |", report.total_failed)?;
    writeln!(f, "| 総文字数 | {} 文字 |", report.total_wordcount)?;
    writeln!(f, "| 総バイト数 | {} バイト |", report.total_bytes)?;
    writeln!(
        f,
        "| 累積CPU時間 | {:.2?} |",
        report.duration_every_thread_total
    )?;
    writeln!(f, "| 実測経過時間 | {:.2?} |\n", report.duration_total)?;

    writeln!(f, "## フェーズ別処理時間\n")?;
    writeln!(f, "| フェーズ | 累積時間 | 平均時間 | 割合 | スループット |")?;
    writeln!(f, "|:---|---:|---:|---:|---:|")?;
    for p in &report.phases {
        writeln!(
            f,
            "| `{}` | {:.2?} | {:.2?} | {:>5.2}% | {:.0} c/s |",
            p.name, p.total, p.avg, p.percentage, p.throughput_chars_per_sec
        )?;
        for sub in &p.subphases {
            writeln!(
                f,
                "| └ `{}` | {:.2?} | {:.2?} | {:>5.2}% | {:.0} c/s |",
                sub.name, sub.total, sub.avg, sub.percentage, sub.throughput_chars_per_sec
            )?;
        }
    }

    writeln!(f, "\n## カテゴリ別内訳\n")?;
    writeln!(f, "| カテゴリ | 累積時間 | 割合 |")?;
    writeln!(f, "|:---|---:|---:|")?;
    for cat in &report.categories {
        writeln!(f, "| {} | {:.2?} | {:>5.1}% |", cat.name, cat.total, cat.percentage)?;
    }

    writeln!(f, "\n## スケーラビリティ指標\n")?;
    writeln!(f, "| 指標 | 計測値 |")?;
    writeln!(f, "|:---|---:|")?;
    writeln!(
        f,
        "| 1,000文字あたりの平均所要時間 | {:.2} µs |",
        report.scaling.us_per_1k_chars_avg
    )?;
    writeln!(
        f,
        "| 1,000文字あたりの最小所要時間 | {:.2} µs |",
        report.scaling.us_per_1k_chars_min
    )?;
    writeln!(
        f,
        "| 1,000文字あたりの最大所要時間 | {:.2} µs |",
        report.scaling.us_per_1k_chars_max
    )?;
    writeln!(
        f,
        "| 1,000文字あたりの純粋パース時間 | {:.2} µs |",
        report.scaling.pure_us_per_1k_chars_avg
    )?;
    writeln!(
        f,
        "| 文字数対パース時間の相関係数 r | {:.4} |",
        report.scaling.linearity_r
    )?;

    if !report.duration_top.is_empty() {
        writeln!(f, "\n## 処理時間上位作品\n")?;
        writeln!(
            f,
            "| 順位 | 作品名 | 著者 | 文字数 | パース時間 | 総処理時間 |"
        )?;
        writeln!(f, "|:---:|:---|:---|---:|---:|---:|")?;
        for (i, w) in report.duration_top.iter().enumerate() {
            if w.title.is_empty() {
                continue;
            }
            writeln!(
                f,
                "| {} | {} | {} | {} 文字 | {:.2?} | {:.2?} |",
                i + 1,
                w.title,
                w.author,
                w.word_count,
                w.total_pure,
                w.total_parsetime
            )?;
        }
    }

    Ok(())
}
