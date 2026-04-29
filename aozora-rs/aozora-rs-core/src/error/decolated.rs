use colored::Colorize;
use memchr::{memchr, memrchr};

use crate::Span;

fn extract_line_begins(original: &str, start_idx: usize) -> usize {
    let bytes = original.as_bytes();
    
    memrchr(b'\n', &bytes[..start_idx])
        .map(|i| i + 1)
        .unwrap_or(0)
}

fn extract_line_ends(original: &str, start_idx: usize) -> usize {
    let bytes = original.as_bytes();
    
    memchr(b'\n', &bytes[start_idx..])
        .map(|i| start_idx + i)
        .unwrap_or(bytes.len())
}

fn count_newlines(text: &str, start_idx: usize, end_idx: usize) -> usize {
    let bytes = text.as_bytes();
    if start_idx >= end_idx || start_idx >= bytes.len() {
        return 0;
    }
    let safe_end = end_idx.min(bytes.len());
    bytes[start_idx..safe_end]
        .iter()
        .filter(|&&b| b == b'\n')
        .count()
}

pub fn display_error_with_decolation(
    original: &str,
    error: Span,
    error_cathegory: &str,
    error_kind: &str,
) -> String {
    let before_err_idx = extract_line_begins(original, error.start);
    let before_err = &original[before_err_idx..error.start];

    let after_err_idx = extract_line_ends(original, error.end);
    let after_err = &original[error.end..after_err_idx];

    let lines = count_newlines(original, 0, before_err_idx);
    let err_colour = "#900";

    format!(
        "{}：{}\n\t{} {} >> {} << {}",
        error_cathegory.color(err_colour),
        error_kind,
        lines.to_string().color("#999"),
        before_err,
        &original[error].color(err_colour),
        after_err
    )
}
