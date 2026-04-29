use std::ops::Range;

use aozora_rs_core::{
    Annotation, AozoraTokenKind, BackRefKind, Deco, MultiLine, PageDef,
    Sandwiched, Scope, Single, Tokenized, WholeLine,
    parse_meta, scopenize, tokenize,
};
use tower_lsp::lsp_types::Position;
use winnow::LocatingSlice;

use crate::line_index::LineIndex;

/// 所有型のメタデータ
pub struct OwnedMeta {
    pub title: String,
    pub author: String,
}

/// 所有型のトークン
pub struct OwnedToken {
    pub kind: OwnedTokenKind,
    pub span: Range<usize>,
}

/// トークン種別の所有型表現
pub enum OwnedTokenKind {
    Annotation(OwnedAnnotation),
    Ruby(String),
    RubyDelimiter,
    Text,
    Br,
}

/// 注記の所有型表現
pub enum OwnedAnnotation {
    BackRef { description: String },
    SandwichedBegin { description: String },
    SandwichedEnd { description: String },
    MultilineBegin { description: String },
    MultilineEnd { description: String },
    Single { description: String },
    WholeLine { description: String },
    PageDef { description: String },
    Unknown(String),
}

/// 所有型のスコープ
pub struct OwnedScope {
    pub deco_description: String,
    pub deco_kind: OwnedDecoKind,
    pub span: Range<usize>,
}

/// 折り畳みや見出し判定に使うDecoの分類
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OwnedDecoKind {
    Block,
    HeadA,
    HeadB,
    HeadC,
    Inline,
}

/// ドキュメント全体の解析状態
pub struct DocumentState {
    pub text: String,
    pub meta: OwnedMeta,
    pub body_offset: usize,
    pub symbol_block: Option<Range<usize>>,
    pub tokens: Vec<OwnedToken>,
    pub scopes: Vec<OwnedScope>,
    pub line_index: LineIndex,
}

fn describe_annotation(annotation: &Annotation<'_>) -> OwnedAnnotation {
    match annotation {
        Annotation::BackRef(b) => {
            let desc = format!("「{}」を{}にします", b.range.0, describe_backref_kind(&b.kind));
            OwnedAnnotation::BackRef { description: desc }
        }
        Annotation::Sandwiched(s) => match s {
            Sandwiched::Begin(b) => {
                let desc = format!("ここから**{}**を適用します", describe_sandwiched_begin(b));
                OwnedAnnotation::SandwichedBegin { description: desc }
            }
            Sandwiched::End(e) => {
                let desc = format!("ここで**{}**を終了します", describe_sandwiched_end(e));
                OwnedAnnotation::SandwichedEnd { description: desc }
            }
        },
        Annotation::Multiline(m) => match m {
            MultiLine::Begin(b) => {
                let desc = format!("ここから**{}**のブロックを開始します", describe_multiline_begin(b));
                OwnedAnnotation::MultilineBegin { description: desc }
            }
            MultiLine::End(e) => {
                let desc = format!("ここで**{}**ブロックを終了します", describe_multiline_end(e));
                OwnedAnnotation::MultilineEnd { description: desc }
            }
        },
        Annotation::Single(s) => {
            let desc = describe_single(s);
            OwnedAnnotation::Single { description: desc }
        }
        Annotation::WholeLine(w) => {
            let desc = describe_wholeline(w);
            OwnedAnnotation::WholeLine { description: desc }
        }
        Annotation::PageDef(p) => {
            let desc = describe_pagedef(p);
            OwnedAnnotation::PageDef { description: desc }
        }
        Annotation::Unknown(s) => {
            OwnedAnnotation::Unknown(s.to_string())
        }
    }
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

fn describe_sandwiched_begin(b: &aozora_rs_core::SandwichedBegins) -> String {
    use aozora_rs_core::SandwichedBegins::*;
    match b {
        BoldBegin => "太字".into(),
        ItalicBegin => "斜体".into(),
        BotenBegin(b) => b.to_string(),
        BosenBegin(b) => b.to_string(),
        AHeadBegin => "大見出し".into(),
        BHeadBegin => "中見出し".into(),
        CHeadBegin => "小見出し".into(),
        SmallerBegin(n) => format!("{}段階小さな文字", n),
        BiggerBegin(n) => format!("{}段階大きな文字", n),
        Warichu => "割り注".into(),
        HorizontalLayout => "横組み".into(),
        Sup => "上付き小文字".into(),
    }
}

fn describe_sandwiched_end(e: &aozora_rs_core::SandwichedEnds) -> String {
    use aozora_rs_core::SandwichedEnds::*;
    match e {
        BoldEnd => "太字".into(),
        ItalicEnd => "斜体".into(),
        BotenEnd(b) => b.to_string(),
        BosenEnd(b) => b.to_string(),
        AHeadEnd => "大見出し".into(),
        BHeadEnd => "中見出し".into(),
        CHeadEnd => "小見出し".into(),
        SmallerEnd => "小さな文字".into(),
        BiggerEnd => "大きな文字".into(),
        WarichuEnd => "割り注".into(),
        HorizontalLayout => "横組み".into(),
        Sup => "上付き小文字".into(),
    }
}

fn describe_multiline_begin(b: &aozora_rs_core::MultiLineBegins) -> String {
    use aozora_rs_core::MultiLineBegins::*;
    match b {
        BlockIndent(bi) => format!("{}字下げ", bi.level),
        HangingIndent(hi) => format!("{}字下げ、折り返して{}字下げ", hi.fst_lvl, hi.snd_lvl),
        Grounded => "地付き".into(),
        LowFlying(l) => format!("地から{}字上げ", l.level),
        Smaller(n) => format!("{}段階小さな文字", n),
        Bigger(n) => format!("{}段階大きな文字", n),
        Kerning(n) => format!("{}字詰め", n),
    }
}

fn describe_multiline_end(e: &aozora_rs_core::MultiLineEnds) -> String {
    use aozora_rs_core::MultiLineEnds::*;
    match e {
        BlockIndentEnd => "字下げ".into(),
        GroundedEnd => "地付き".into(),
        LowFlyingEnd => "字上げ".into(),
        SmallEnd => "小さな文字".into(),
        BigEnd => "大きな文字".into(),
        Kerning => "字詰め".into(),
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

fn convert_token(token: &Tokenized<'_>) -> OwnedToken {
    let kind = match &token.kind {
        AozoraTokenKind::Annotation(a) => OwnedTokenKind::Annotation(describe_annotation(a)),
        AozoraTokenKind::Ruby(r) => OwnedTokenKind::Ruby(r.to_string()),
        AozoraTokenKind::RubyDelimiter => OwnedTokenKind::RubyDelimiter,
        AozoraTokenKind::Text(_) => OwnedTokenKind::Text,
        AozoraTokenKind::Br => OwnedTokenKind::Br,
    };
    OwnedToken {
        kind,
        span: token.span.clone(),
    }
}

fn classify_deco(deco: &Deco<'_>) -> OwnedDecoKind {
    match deco {
        Deco::Indent(_) | Deco::Hanging(_) | Deco::Grounded | Deco::LowFlying(_)
        | Deco::Smaller(_) | Deco::Bigger(_) | Deco::Kerning(_) => OwnedDecoKind::Block,
        Deco::AHead => OwnedDecoKind::HeadA,
        Deco::BHead => OwnedDecoKind::HeadB,
        Deco::CHead => OwnedDecoKind::HeadC,
        _ => OwnedDecoKind::Inline,
    }
}

fn convert_scope(scope: &Scope<'_>) -> OwnedScope {
    OwnedScope {
        deco_description: scope.deco.to_string(),
        deco_kind: classify_deco(&scope.deco),
        span: scope.span.clone(),
    }
}

fn detect_symbol_block(text: &str, body_offset: usize) -> Option<Range<usize>> {
    let header = &text[..body_offset];
    let separator = "-------------------------------------------------------";
    let first = header.find(separator)?;
    let after_first = first + separator.len();
    let second = header[after_first..].find(separator)?;
    let end = after_first + second + separator.len();
    Some(first..end)
}

impl DocumentState {
    /// テキストを解析してDocumentStateを構築する。
    /// メタデータ解析に失敗した場合はNoneを返す。
    pub fn parse(text: String) -> Option<Self> {
        let line_index = LineIndex::new(&text);

        let mut cursor = text.as_str();
        let meta = parse_meta(&mut cursor).ok()?;
        let body_offset = text.len() - cursor.len();

        let owned_meta = OwnedMeta {
            title: meta.title.to_string(),
            author: meta.author.to_string(),
        };

        let symbol_block = detect_symbol_block(&text, body_offset);

        let mut loc = LocatingSlice::new(cursor);
        let tokenized = tokenize(&mut loc).ok()?;

        let owned_tokens: Vec<OwnedToken> = tokenized
            .iter()
            .map(|t| {
                let mut ot = convert_token(t);
                ot.span = (ot.span.start + body_offset)..(ot.span.end + body_offset);
                ot
            })
            .collect();

        let ((scopes, _expressions), _errors) = scopenize(tokenized).into_tuple();

        let owned_scopes: Vec<OwnedScope> = scopes
            .iter()
            .map(|s| {
                let mut os = convert_scope(s);
                os.span = (os.span.start + body_offset)..(os.span.end + body_offset);
                os
            })
            .collect();

        Some(DocumentState {
            text,
            meta: owned_meta,
            body_offset,
            symbol_block,
            tokens: owned_tokens,
            scopes: owned_scopes,
            line_index,
        })
    }

    pub fn token_at_offset(&self, offset: usize) -> Option<&OwnedToken> {
        self.tokens
            .iter()
            .find(|t| t.span.start <= offset && offset < t.span.end)
    }

    pub fn scopes_at_offset(&self, offset: usize) -> Vec<&OwnedScope> {
        self.scopes
            .iter()
            .filter(|s| s.span.start <= offset && offset < s.span.end)
            .collect()
    }

    pub fn offset_at_position(&self, pos: Position) -> usize {
        self.line_index.position_to_offset(&self.text, pos)
    }
}
