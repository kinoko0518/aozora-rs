use tower_lsp::lsp_types::{
    SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokens,
    SemanticTokensLegend,
};

use crate::document::{DocumentState, OwnedAnnotation, OwnedTokenKind};

/// LSPに公開するセマンティックトークンタイプ一覧
pub const TOKEN_TYPES: &[SemanticTokenType] = &[
    SemanticTokenType::MACRO,     // 0: 注記
    SemanticTokenType::STRING,    // 1: ルビ
    SemanticTokenType::OPERATOR,  // 2: ルビ区切り
    SemanticTokenType::NAMESPACE, // 3: タイトル
    SemanticTokenType::TYPE,      // 4: 著者
    SemanticTokenType::COMMENT,   // 5: 記号説明ブロック
];

/// LSPに公開するセマンティックトークン修飾子一覧
pub const TOKEN_MODIFIERS: &[SemanticTokenModifier] = &[
    SemanticTokenModifier::DECLARATION,  // 0: 開始注記
    SemanticTokenModifier::MODIFICATION, // 1: 終了注記
    SemanticTokenModifier::DEPRECATED,   // 2: 未知の注記
];

pub fn legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: TOKEN_TYPES.to_vec(),
        token_modifiers: TOKEN_MODIFIERS.to_vec(),
    }
}

/// ドキュメント全体のセマンティックトークンを生成する
pub fn compute_semantic_tokens(doc: &DocumentState) -> SemanticTokens {
    let mut tokens = Vec::new();

    // メタデータ部分のトークン
    emit_metadata_tokens(doc, &mut tokens);

    // 本文のトークン
    for token in &doc.tokens {
        let (token_type, modifiers) = match &token.kind {
            OwnedTokenKind::Annotation(a) => {
                let modifier = match a {
                    OwnedAnnotation::SandwichedBegin { .. }
                    | OwnedAnnotation::MultilineBegin { .. } => 1u32 << 0, // declaration
                    OwnedAnnotation::SandwichedEnd { .. }
                    | OwnedAnnotation::MultilineEnd { .. } => 1u32 << 1, // modification
                    OwnedAnnotation::Unknown(_) => 1u32 << 2,            // deprecated
                    _ => 0,
                };
                (0u32, modifier) // macro
            }
            OwnedTokenKind::Ruby(_) => (1, 0),          // string
            OwnedTokenKind::RubyDelimiter => (2, 0),     // operator
            OwnedTokenKind::Text | OwnedTokenKind::Br => continue,
        };

        let start = doc.line_index.offset_to_position(&doc.text, token.span.start);
        let end = doc.line_index.offset_to_position(&doc.text, token.span.end);

        // セマンティックトークンが複数行にまたがる場合、行ごとに分割する
        if start.line == end.line {
            tokens.push(RawSemanticToken {
                line: start.line,
                start_char: start.character,
                length: end.character - start.character,
                token_type,
                modifiers,
            });
        } else {
            // 注記が複数行にまたがることは稀だが対応する
            // 先頭行
            let first_line_end = doc
                .text[token.span.start..]
                .find('\n')
                .map(|i| token.span.start + i)
                .unwrap_or(token.span.end);
            let first_end = doc.line_index.offset_to_position(&doc.text, first_line_end);
            tokens.push(RawSemanticToken {
                line: start.line,
                start_char: start.character,
                length: first_end.character - start.character,
                token_type,
                modifiers,
            });

            // 中間行と最終行
            let mut current_offset = first_line_end + 1;
            while current_offset < token.span.end {
                let line_end = doc.text[current_offset..]
                    .find('\n')
                    .map(|i| current_offset + i)
                    .unwrap_or(token.span.end);
                let actual_end = line_end.min(token.span.end);
                let line_start_pos = doc.line_index.offset_to_position(&doc.text, current_offset);
                let line_end_pos = doc.line_index.offset_to_position(&doc.text, actual_end);
                if line_end_pos.character > 0 {
                    tokens.push(RawSemanticToken {
                        line: line_start_pos.line,
                        start_char: 0,
                        length: line_end_pos.character,
                        token_type,
                        modifiers,
                    });
                }
                current_offset = line_end + 1;
            }
        }
    }

    // 行・位置でソート
    tokens.sort_by(|a, b| a.line.cmp(&b.line).then(a.start_char.cmp(&b.start_char)));

    // デルタエンコーディング
    let mut prev_line = 0u32;
    let mut prev_start = 0u32;
    let data: Vec<SemanticToken> = tokens
        .iter()
        .map(|t| {
            let delta_line = t.line - prev_line;
            let delta_start = if delta_line == 0 {
                t.start_char - prev_start
            } else {
                t.start_char
            };
            prev_line = t.line;
            prev_start = t.start_char;
            SemanticToken {
                delta_line,
                delta_start,
                length: t.length,
                token_type: t.token_type,
                token_modifiers_bitset: t.modifiers,
            }
        })
        .collect();

    SemanticTokens {
        result_id: None,
        data,
    }
}

struct RawSemanticToken {
    line: u32,
    start_char: u32,
    length: u32,
    token_type: u32,
    modifiers: u32,
}

/// メタデータ領域のセマンティックトークンを出力する
fn emit_metadata_tokens(doc: &DocumentState, tokens: &mut Vec<RawSemanticToken>) {
    let header = &doc.text[..doc.body_offset];

    // タイトル行 (1行目)
    if let Some(newline_pos) = header.find('\n') {
        let title_end = doc.line_index.offset_to_position(&doc.text, newline_pos);
        tokens.push(RawSemanticToken {
            line: 0,
            start_char: 0,
            length: title_end.character,
            token_type: 3, // namespace
            modifiers: 0,
        });

        // 著者行 (2行目)
        let author_start = newline_pos + 1;
        if let Some(second_newline) = header[author_start..].find('\n') {
            let author_end_offset = author_start + second_newline;
            let author_start_pos = doc.line_index.offset_to_position(&doc.text, author_start);
            let author_end_pos = doc.line_index.offset_to_position(&doc.text, author_end_offset);
            tokens.push(RawSemanticToken {
                line: author_start_pos.line,
                start_char: 0,
                length: author_end_pos.character,
                token_type: 4, // type
                modifiers: 0,
            });
        }
    }

    // 記号説明ブロック
    if let Some(ref block) = doc.symbol_block {
        let block_text = &doc.text[block.start..block.end];
        let mut offset = block.start;
        for line in block_text.split('\n') {
            if !line.is_empty() {
                let pos = doc.line_index.offset_to_position(&doc.text, offset);
                let line_utf16_len = line.trim_end_matches('\r').encode_utf16().count() as u32;
                if line_utf16_len > 0 {
                    tokens.push(RawSemanticToken {
                        line: pos.line,
                        start_char: 0,
                        length: line_utf16_len,
                        token_type: 5, // comment
                        modifiers: 0,
                    });
                }
            }
            offset += line.len() + 1; // +1 for '\n'
        }
    }
}
