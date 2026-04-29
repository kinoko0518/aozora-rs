use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position};

use crate::document::{DocumentState, OwnedAnnotation, OwnedTokenKind};

/// カーソル位置に応じたホバー情報を生成する
pub fn compute_hover(doc: &DocumentState, pos: Position) -> Option<Hover> {
    let offset = doc.offset_at_position(pos);

    // メタデータ領域のホバー
    if offset < doc.body_offset {
        return hover_metadata(doc, offset);
    }

    // トークン上のホバー
    if let Some(token) = doc.token_at_offset(offset) {
        return match &token.kind {
            OwnedTokenKind::Annotation(a) => Some(hover_annotation(a)),
            OwnedTokenKind::Ruby(r) => Some(hover_ruby(doc, r, offset)),
            OwnedTokenKind::RubyDelimiter => Some(simple_hover(
                "**ルビ区切り** `｜`\n\nこの後のテキストにルビの適用範囲を明示します。",
            )),
            OwnedTokenKind::Text => hover_text_decorations(doc, offset),
            OwnedTokenKind::Br => None,
        };
    }

    // テキストトークンに含まれない領域でもスコープがあれば表示
    hover_text_decorations(doc, offset)
}

fn hover_metadata(doc: &DocumentState, offset: usize) -> Option<Hover> {
    let header = &doc.text[..doc.body_offset];

    // タイトル行
    if let Some(first_newline) = header.find('\n') {
        if offset <= first_newline {
            return Some(simple_hover(&format!(
                "### タイトル\n**{}**",
                doc.meta.title
            )));
        }
        // 著者行
        let author_start = first_newline + 1;
        if let Some(second_newline) = header[author_start..].find('\n')
            && offset >= author_start && offset <= author_start + second_newline {
                return Some(simple_hover(&format!("### 著者\n**{}**", doc.meta.author)));
            }
    }

    // 記号説明ブロック
    if let Some(ref block) = doc.symbol_block
        && offset >= block.start && offset < block.end {
            return Some(simple_hover(
                "### テキスト中に現れる記号について\n\nこのブロックではテキスト内で使われる注記記号の説明が記載されています。\n出力には含まれません。",
            ));
        }

    None
}

fn hover_annotation(a: &OwnedAnnotation) -> Hover {
    let (category, description) = match a {
        OwnedAnnotation::BackRef { description } => ("前方参照型注記", description.as_str()),
        OwnedAnnotation::SandwichedBegin { description } => {
            ("行内挟み込み型注記（開始）", description.as_str())
        }
        OwnedAnnotation::SandwichedEnd { description } => {
            ("行内挟み込み型注記（終了）", description.as_str())
        }
        OwnedAnnotation::MultilineBegin { description } => {
            ("複数行挟み込み型注記（開始）", description.as_str())
        }
        OwnedAnnotation::MultilineEnd { description } => {
            ("複数行挟み込み型注記（終了）", description.as_str())
        }
        OwnedAnnotation::Single { description } => ("単体注記", description.as_str()),
        OwnedAnnotation::WholeLine { description } => ("行頭型注記", description.as_str()),
        OwnedAnnotation::PageDef { description } => ("ページ定義注記", description.as_str()),
        OwnedAnnotation::Unknown(s) => {
            return simple_hover(&format!(
                "### ⚠ 不明な注記\n\n`{}` はaozora-rsが認識できない注記です。",
                s
            ));
        }
    };
    simple_hover(&format!(
        "### 注記\n**種別**: {}\n\n{}",
        category, description
    ))
}

fn hover_ruby(doc: &DocumentState, ruby_text: &str, _offset: usize) -> Hover {
    // スコープからルビの対象テキストを逆引きする
    let target = doc
        .scopes
        .iter()
        .find(|s| s.deco_description.contains("ルビ") && s.deco_description.contains(ruby_text))
        .map(|s| &doc.text[s.span.clone()])
        .unwrap_or("（対象不明）");

    simple_hover(&format!(
        "### ルビ\n**対象**: {}\n**読み**: {}",
        target, ruby_text
    ))
}

fn hover_text_decorations(doc: &DocumentState, offset: usize) -> Option<Hover> {
    let scopes = doc.scopes_at_offset(offset);
    if scopes.is_empty() {
        return None;
    }

    let mut lines = vec!["### 適用中の装飾".to_string()];
    for scope in &scopes {
        lines.push(format!("- {}", scope.deco_description));
    }
    Some(simple_hover(&lines.join("\n")))
}

fn simple_hover(markdown: &str) -> Hover {
    Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: markdown.to_string(),
        }),
        range: None,
    }
}
