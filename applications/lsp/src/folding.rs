use tower_lsp::lsp_types::{FoldingRange, FoldingRangeKind};

use crate::document::{DocumentState, OwnedDecoKind};

/// ドキュメントの折り畳み範囲を計算する
pub fn compute_folding_ranges(doc: &DocumentState) -> Vec<FoldingRange> {
    let mut ranges = Vec::new();

    // 記号説明ブロックの折り畳み
    if let Some(ref block) = doc.symbol_block {
        let start = doc.line_index.offset_to_position(&doc.text, block.start);
        let end = doc.line_index.offset_to_position(&doc.text, block.end);
        if end.line > start.line {
            ranges.push(FoldingRange {
                start_line: start.line,
                start_character: None,
                end_line: end.line,
                end_character: None,
                kind: Some(FoldingRangeKind::Comment),
                collapsed_text: Some("【テキスト中に現れる記号について】".to_string()),
            });
        }
    }

    // 複数行ブロック注記の折り畳み
    for scope in &doc.scopes {
        if scope.deco_kind != OwnedDecoKind::Block {
            continue;
        }
        let start = doc.line_index.offset_to_position(&doc.text, scope.span.start);
        let end = doc.line_index.offset_to_position(&doc.text, scope.span.end);
        if end.line > start.line {
            ranges.push(FoldingRange {
                start_line: start.line,
                start_character: None,
                end_line: end.line,
                end_character: None,
                kind: Some(FoldingRangeKind::Region),
                collapsed_text: Some(scope.deco_description.clone()),
            });
        }
    }

    // 見出しベースのセクション折り畳み
    compute_heading_folds(doc, &mut ranges);

    ranges
}

/// 見出しの階層構造に基づいてセクション折り畳みを生成する
fn compute_heading_folds(doc: &DocumentState, ranges: &mut Vec<FoldingRange>) {
    // 見出しスコープを位置順に収集
    let mut headings: Vec<(u32, OwnedDecoKind)> = doc
        .scopes
        .iter()
        .filter(|s| matches!(s.deco_kind, OwnedDecoKind::HeadA | OwnedDecoKind::HeadB | OwnedDecoKind::HeadC))
        .map(|s| {
            let line = doc.line_index.offset_to_position(&doc.text, s.span.start).line;
            (line, s.deco_kind)
        })
        .collect();

    headings.sort_by_key(|(line, _)| *line);

    if headings.is_empty() {
        return;
    }

    fn heading_level(kind: OwnedDecoKind) -> u8 {
        match kind {
            OwnedDecoKind::HeadA => 1,
            OwnedDecoKind::HeadB => 2,
            OwnedDecoKind::HeadC => 3,
            _ => unreachable!(),
        }
    }

    let total_lines = doc.line_index.offset_to_position(&doc.text, doc.text.len()).line;

    for (i, (line, kind)) in headings.iter().enumerate() {
        let level = heading_level(*kind);

        // 次の同レベル以上の見出し、またはドキュメント末を探す
        let end_line = headings[i + 1..]
            .iter()
            .find(|(_, k)| heading_level(*k) <= level)
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
