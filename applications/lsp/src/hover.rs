use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position};

use aozora_rs_core::{
    Annotation, AozoraTokenKind, BackRefKind, MultiLineBegins, MultiLineEnds, PageDef, SandwichedBegins,
    SandwichedEnds, Single, WholeLine, Sandwiched, MultiLine,
};
use crate::document::DocumentState;

/// カーソル位置に応じたホバー情報を生成する
pub fn compute_hover(doc: &DocumentState, pos: Position) -> Option<Hover> {
    let offset = doc.offset_at_position(pos);

    // メタデータ領域のホバー
    if offset < doc.parsed().body_offset {
        return hover_metadata(doc, offset);
    }

    // トークン上のホバー
    if let Some(token) = doc.token_at_offset(offset) {
        return match &token.kind {
            AozoraTokenKind::Annotation(a) => Some(hover_annotation(a)),
            AozoraTokenKind::Ruby(r) => Some(hover_ruby(doc, r, offset)),
            AozoraTokenKind::RubyDelimiter => Some(simple_hover(
                "**ルビ区切り** `｜`\n\nこの後のテキストにルビの適用範囲を明示します。",
            )),
            AozoraTokenKind::Text(_) => hover_text_decorations(doc, offset),
            AozoraTokenKind::Br => None,
        };
    }

    // テキストトークンに含まれない領域でもスコープがあれば表示
    hover_text_decorations(doc, offset)
}

fn hover_metadata(doc: &DocumentState, offset: usize) -> Option<Hover> {
    let header = &doc.text()[..doc.parsed().body_offset];

    // タイトル行
    if let Some(first_newline) = header.find('\n') {
        if offset <= first_newline {
            return Some(simple_hover(&format!(
                "### タイトル\n**{}**",
                doc.parsed().meta.as_ref().map_or("", |m| m.title)
            )));
        }
        // 著者行
        let author_start = first_newline + 1;
        if let Some(second_newline) = header[author_start..].find('\n') {
            if offset >= author_start && offset <= author_start + second_newline {
                return Some(simple_hover(&format!(
                    "### 著者\n**{}**",
                    doc.parsed().meta.as_ref().map_or("", |m| m.author)
                )));
            }
        }
    }

    // 記号説明ブロック
    if let Some(ref block) = doc.parsed().symbol_block {
        if offset >= block.start && offset < block.end {
            return Some(simple_hover(
                "### テキスト中に現れる記号について\n\nこのブロックではテキスト内で使われる注記記号の説明が記載されています。\n出力には含まれません。",
            ));
        }
    }

    None
}

fn hover_annotation(a: &Annotation<'_>) -> Hover {
    let (category, description) = match a {
        Annotation::BackRef(b) => (
            "前方参照型注記",
            format!("「{}」を{}にします", b.range.0, describe_backref_kind(&b.kind)),
        ),
        Annotation::Sandwiched(Sandwiched::Begin(b)) => (
            "行内挟み込み型注記（開始）",
            format!("ここから**{}**を適用します", describe_sandwiched_begin(b)),
        ),
        Annotation::Sandwiched(Sandwiched::End(e)) => (
            "行内挟み込み型注記（終了）",
            format!("ここで**{}**を終了します", describe_sandwiched_end(e)),
        ),
        Annotation::Multiline(MultiLine::Begin(b)) => (
            "複数行挟み込み型注記（開始）",
            format!("ここから**{}**のブロックを開始します", describe_multiline_begin(b)),
        ),
        Annotation::Multiline(MultiLine::End(e)) => (
            "複数行挟み込み型注記（終了）",
            format!("ここで**{}**ブロックを終了します", describe_multiline_end(e)),
        ),
        Annotation::Single(s) => ("単体注記", describe_single(s)),
        Annotation::WholeLine(w) => ("行頭型注記", describe_wholeline(w)),
        Annotation::PageDef(p) => ("ページ定義注記", describe_pagedef(p)),
        Annotation::Unknown(s) => {
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

fn describe_backref_kind(kind: &BackRefKind<'_>) -> String {
    match kind {
        BackRefKind::Bold => "太字".into(),
        BackRefKind::Italic => "斜体".into(),
        BackRefKind::Boten(b) => b.to_string(),
        BackRefKind::Bosen(b) => b.to_string(),
        BackRefKind::AHead => "大見出し".into(),
        BackRefKind::BHead => "中見出し".into(),
        BackRefKind::CHead => "小見出し".into(),
        BackRefKind::Mama => "ママ".into(),
        BackRefKind::HinV => "縦中横".into(),
        BackRefKind::Small(n) => format!("{}段階小さな文字", n),
        BackRefKind::Big(n) => format!("{}段階大きな文字", n),
        BackRefKind::Note(n) => format!("「{}」の注記", n),
        BackRefKind::Variation((on, v)) => format!("{}では「{}」", on, v),
        BackRefKind::Sub => "下付き小文字".into(),
        BackRefKind::Sup => "上付き小文字".into(),
    }
}

fn describe_sandwiched_begin(b: &SandwichedBegins) -> String {
    match b {
        SandwichedBegins::BoldBegin => "太字".into(),
        SandwichedBegins::ItalicBegin => "斜体".into(),
        SandwichedBegins::BotenBegin(b) => b.to_string(),
        SandwichedBegins::BosenBegin(b) => b.to_string(),
        SandwichedBegins::AHeadBegin => "大見出し".into(),
        SandwichedBegins::BHeadBegin => "中見出し".into(),
        SandwichedBegins::CHeadBegin => "小見出し".into(),
        SandwichedBegins::SmallerBegin(n) => format!("{}段階小さな文字", n),
        SandwichedBegins::BiggerBegin(n) => format!("{}段階大きな文字", n),
        SandwichedBegins::Warichu => "割り注".into(),
        SandwichedBegins::HorizontalLayout => "横組み".into(),
        SandwichedBegins::Sup => "上付き小文字".into(),
    }
}

fn describe_sandwiched_end(e: &SandwichedEnds) -> String {
    match e {
        SandwichedEnds::BoldEnd => "太字".into(),
        SandwichedEnds::ItalicEnd => "斜体".into(),
        SandwichedEnds::BotenEnd(b) => b.to_string(),
        SandwichedEnds::BosenEnd(b) => b.to_string(),
        SandwichedEnds::AHeadEnd => "大見出し".into(),
        SandwichedEnds::BHeadEnd => "中見出し".into(),
        SandwichedEnds::CHeadEnd => "小見出し".into(),
        SandwichedEnds::SmallerEnd => "小さな文字".into(),
        SandwichedEnds::BiggerEnd => "大きな文字".into(),
        SandwichedEnds::WarichuEnd => "割り注".into(),
        SandwichedEnds::HorizontalLayout => "横組み".into(),
        SandwichedEnds::Sup => "上付き小文字".into(),
    }
}

fn describe_multiline_begin(b: &MultiLineBegins) -> String {
    match b {
        MultiLineBegins::BlockIndent(bi) => format!("{}字下げ", bi.level),
        MultiLineBegins::HangingIndent(hi) => format!("{}字下げ、折り返して{}字下げ", hi.fst_lvl, hi.snd_lvl),
        MultiLineBegins::Grounded => "地付き".into(),
        MultiLineBegins::LowFlying(l) => format!("地から{}字上げ", l.level),
        MultiLineBegins::Smaller(n) => format!("{}段階小さな文字", n),
        MultiLineBegins::Bigger(n) => format!("{}段階大きな文字", n),
        MultiLineBegins::Kerning(n) => format!("{}字詰め", n),
    }
}

fn describe_multiline_end(e: &MultiLineEnds) -> String {
    match e {
        MultiLineEnds::BlockIndentEnd => "字下げ".into(),
        MultiLineEnds::GroundedEnd => "地付き".into(),
        MultiLineEnds::LowFlyingEnd => "字上げ".into(),
        MultiLineEnds::SmallEnd => "小さな文字".into(),
        MultiLineEnds::BigEnd => "大きな文字".into(),
        MultiLineEnds::Kerning => "字詰め".into(),
    }
}

fn describe_single(s: &Single<'_>) -> String {
    match s {
        Single::PageBreak => "ここで**改ページ**します".into(),
        Single::RectoBreak => "ここで**改丁**します（左ページから再開）".into(),
        Single::SpreadBreak => "ここで**改見開き**します（右ページから再開）".into(),
        Single::ColumnBreak => "ここで**改段**します".into(),
        Single::Figure(f) => format!("図: **{}**（{}）", f.caption, f.path),
        Single::Kundoku(k) => format!("訓点「{}」", k),
        Single::Okurigana(o) => format!("送り仮名「{}」", o),
    }
}

fn describe_wholeline(w: &WholeLine) -> String {
    match w {
        WholeLine::Indent(n) => format!("この行を**{}字下げ**します", n),
        WholeLine::Grounded => "この行を**地付き**（右端揃え）にします".into(),
        WholeLine::LowFlying(n) => format!("この行を**地から{}字上げ**します", n),
    }
}

fn describe_pagedef(p: &PageDef) -> String {
    match p {
        PageDef::VHCentre => "ページを**左右中央**に配置します".into(),
        PageDef::FromLeft => "左ページから開始します".into(),
        PageDef::FromRight => "右ページから開始します".into(),
    }
}

fn hover_ruby(doc: &DocumentState, ruby_text: &str, _offset: usize) -> Hover {
    // スコープからルビの対象テキストを逆引きする
    let target = doc.parsed()
        .scopes
        .iter()
        .find(|s| s.deco.to_string().contains("ルビ") && s.deco.to_string().contains(ruby_text))
        .map(|s| &doc.text()[s.span.clone()])
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
        lines.push(format!("- {}", scope.deco.to_string()));
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
