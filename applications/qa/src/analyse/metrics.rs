use std::cmp::Ordering;
use std::time::Duration;
use serde::Serialize;

use super::per_work::WorkAnalyse;

pub const RANKING_LEN: usize = 10;

#[derive(Debug, Clone, Serialize)]
pub struct PhaseMetric {
    pub name: String,
    pub total: Duration,
    pub avg: Duration,
    pub percentage: f64,
    pub throughput_chars_per_sec: f64,
    pub subphases: Vec<PhaseMetric>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CategoryMetric {
    pub name: String,
    pub total: Duration,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScalingMetric {
    pub us_per_1k_chars_avg: f64,
    pub us_per_1k_chars_min: f64,
    pub us_per_1k_chars_max: f64,
    pub pure_us_per_1k_chars_avg: f64,
    pub linearity_r: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PerformanceReport {
    pub total_works: usize,
    pub total_succeed: usize,
    pub total_failed: usize,
    pub total_wordcount: usize,
    pub total_bytes: usize,
    pub scopenize_warning_total: usize,
    pub retokenize_warning_total: usize,
    pub duration_every_thread_total: Duration,
    pub duration_total: Duration,
    pub phases: Vec<PhaseMetric>,
    pub categories: Vec<CategoryMetric>,
    pub scaling: ScalingMetric,
    pub duration_top: Vec<WorkAnalyse>,
}

pub fn compute_performance_report(
    works: &[WorkAnalyse],
    total_works: usize,
    total_failed: usize,
    elapsed_wall_clock: Duration,
) -> PerformanceReport {
    let count = works.len().max(1) as f64;
    let total_duration: Duration = works.iter().map(|w| w.total_parsetime).sum();
    let total_secs = total_duration.as_secs_f64().max(1e-9);
    let total_wordcount: usize = works.iter().map(|w| w.word_count).sum();
    let total_bytes: usize = works.iter().map(|w| w.byte_count).sum();
    let scopenize_warning_total: usize = works.iter().map(|w| w.scopenize_errors.len()).sum();
    let retokenize_warning_total: usize = works.iter().map(|w| w.retokenize_errors.len()).sum();

    let calc_phase = |name: &str, get_duration: fn(&WorkAnalyse) -> Duration| -> PhaseMetric {
        let phase_total: Duration = works.iter().map(get_duration).sum();
        let avg = phase_total.div_f64(count);
        let percentage = (phase_total.as_secs_f64() / total_secs) * 100.0;
        let throughput = if phase_total.as_secs_f64() > 0.0 {
            total_wordcount as f64 / phase_total.as_secs_f64()
        } else {
            0.0
        };
        PhaseMetric {
            name: name.to_string(),
            total: phase_total,
            avg,
            percentage,
            throughput_chars_per_sec: throughput,
            subphases: Vec::new(),
        }
    };

    let mut read_metric = calc_phase("read", |w| w.read);
    read_metric.subphases = vec![
        calc_phase("read_io", |w| w.read_io),
        calc_phase("read_decode", |w| w.read_decode),
    ];

    let phases = vec![
        read_metric,
        calc_phase("gaiji_convert", |w| w.gaiji_convert),
        calc_phase("get_meta", |w| w.get_meta),
        calc_phase("tokenize", |w| w.tokenize),
        calc_phase("scopenize", |w| w.scopenize),
        calc_phase("retokenize", |w| w.retokenize),
        calc_phase("xhtml_gen", |w| w.xhtml_gen),
        calc_phase("epub_gen", |w| w.epub_gen),
    ];

    // カテゴリ別集計
    let pure_parser_total: Duration = works.iter().map(|w| w.total_pure).sum();
    let pure_parser_percentage = (pure_parser_total.as_secs_f64() / total_secs) * 100.0;

    let xhtml_total: Duration = works.iter().map(|w| w.xhtml_gen).sum();
    let xhtml_percentage = (xhtml_total.as_secs_f64() / total_secs) * 100.0;

    let io_and_epub_total: Duration = works.iter().map(|w| w.read + w.epub_gen).sum();
    let io_and_epub_percentage = (io_and_epub_total.as_secs_f64() / total_secs) * 100.0;

    let categories = vec![
        CategoryMetric {
            name: "構文解析".to_string(),
            total: pure_parser_total,
            percentage: pure_parser_percentage,
        },
        CategoryMetric {
            name: "XHTML生成".to_string(),
            total: xhtml_total,
            percentage: xhtml_percentage,
        },
        CategoryMetric {
            name: "I/OおよびEPUB生成".to_string(),
            total: io_and_epub_total,
            percentage: io_and_epub_percentage,
        },
    ];

    // スケーラビリティ指標
    let n = works.len() as f64;
    let linearity_r = if n > 1.0 {
        let (sum_x, sum_y, sum_xy, sum_x2, sum_y2) = works.iter().fold(
            (0.0, 0.0, 0.0, 0.0, 0.0),
            |(sx, sy, sxy, sx2, sy2), w| {
                let x = w.word_count as f64;
                let y = w.total_pure.as_micros() as f64;
                (sx + x, sy + y, sxy + x * y, sx2 + x * x, sy2 + y * y)
            },
        );
        let denom = ((n * sum_x2 - sum_x * sum_x) * (n * sum_y2 - sum_y * sum_y)).sqrt();
        if denom > 0.0 {
            (n * sum_xy - sum_x * sum_y) / denom
        } else {
            1.0
        }
    } else {
        1.0
    };

    let mut us_per_1k: Vec<f64> = works
        .iter()
        .filter(|w| w.word_count > 0)
        .map(|w| (w.total_parsetime.as_micros() as f64 / w.word_count as f64) * 1000.0)
        .collect();
    us_per_1k.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

    let us_per_1k_avg = if total_wordcount > 0 {
        (total_duration.as_micros() as f64 / total_wordcount as f64) * 1000.0
    } else {
        0.0
    };
    let pure_us_per_1k_avg = if total_wordcount > 0 {
        (pure_parser_total.as_micros() as f64 / total_wordcount as f64) * 1000.0
    } else {
        0.0
    };

    let scaling = ScalingMetric {
        us_per_1k_chars_avg: us_per_1k_avg,
        us_per_1k_chars_min: us_per_1k.first().copied().unwrap_or(0.0),
        us_per_1k_chars_max: us_per_1k.last().copied().unwrap_or(0.0),
        pure_us_per_1k_chars_avg: pure_us_per_1k_avg,
        linearity_r,
    };

    // 所要時間ランキング
    let mut sorted_works = works.to_vec();
    sorted_works.sort_by_key(|b| std::cmp::Reverse(b.total_parsetime));
    let duration_top = sorted_works.into_iter().take(RANKING_LEN).collect();

    PerformanceReport {
        total_works,
        total_succeed: works.len(),
        total_failed,
        total_wordcount,
        total_bytes,
        scopenize_warning_total,
        retokenize_warning_total,
        duration_every_thread_total: total_duration,
        duration_total: elapsed_wall_clock,
        phases,
        categories,
        scaling,
        duration_top,
    }
}
