use tower_lsp::lsp_types::{FoldingRange, FoldingRangeKind};

use crate::document::DocumentState;
use aozora_rs_core::Deco;

/// ドキュメントの折り畳み範囲を計算する
pub fn compute_folding_ranges(doc: &DocumentState) -> Vec<FoldingRange> {
    let mut ranges = Vec::new();
    compute_heading_folds(doc, &mut ranges);
    ranges
}

/// 見出しの階層構造に基づいてセクション折り畳みを生成する
fn compute_heading_folds(doc: &DocumentState, ranges: &mut Vec<FoldingRange>) {
    // 見出しスコープを位置順に収集
    let mut headings: Vec<(u32, u8)> = doc
        .parsed()
        .scopes
        .iter()
        .filter_map(|s| {
            let level = match s.deco {
                Deco::AHead => 1,
                Deco::BHead => 2,
                Deco::CHead => 3,
                _ => return None,
            };
            let line = doc
                .line_index
                .offset_to_position(doc.text(), s.span.start)
                .line;
            Some((line, level))
        })
        .collect();

    headings.sort_by_key(|(line, _)| *line);

    if headings.is_empty() {
        return;
    }

    let total_lines = doc
        .line_index
        .offset_to_position(doc.text(), doc.text().len())
        .line;

    for (i, (line, level)) in headings.iter().enumerate() {
        // 次の同レベル以上の見出し、またはドキュメント末を探す
        let end_line = headings[i + 1..]
            .iter()
            .find(|(_, k)| k <= level)
            .map(|(l, _)| l.saturating_sub(1))
            .unwrap_or(total_lines);

        if end_line > *line {
            ranges.push(FoldingRange {
                start_line: *line,
                start_character: None,
                end_line,
                end_character: None,
                kind: Some(FoldingRangeKind::Region),
                collapsed_text: None,
            });
        }
    }
}
