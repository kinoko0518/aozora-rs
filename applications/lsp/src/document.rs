use std::ops::Range;

use aozora_rs_core::{
    parse_meta, scopenize, tokenize, AozoraMeta, Scope, ScopenizeError, Tokenized,
};
use ouroboros::self_referencing;
use tower_lsp::lsp_types::Position;
use winnow::LocatingSlice;

use crate::line_index::LineIndex;

pub struct ParsedData<'a> {
    pub meta: Option<AozoraMeta<'a>>,
    pub body_offset: usize,
    pub symbol_block: Option<Range<usize>>,
    pub tokens: Vec<Tokenized<'a>>,
    pub scopes: Vec<Scope<'a>>,
    pub errors: Vec<ScopenizeError>,
}

#[self_referencing]
pub struct DocumentStateInner {
    pub text: String,

    #[borrows(text)]
    #[covariant]
    pub parsed: ParsedData<'this>,
}

pub struct DocumentState {
    pub inner: DocumentStateInner,
    pub line_index: LineIndex,
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
    pub fn parse(text: String) -> Self {
        let line_index = LineIndex::new(&text);

        let inner = DocumentStateInnerBuilder {
            text,
            parsed_builder: |text| {
                let mut cursor = text.as_str();
                let meta = parse_meta(&mut cursor).ok();
                let body_offset = text.len() - cursor.len();

                let symbol_block = detect_symbol_block(text, body_offset);

                let mut loc = LocatingSlice::new(cursor);
                let tokens = tokenize(&mut loc).unwrap_or_default();

                let ((mut scopes, _expressions), mut errors) =
                    scopenize(tokens.clone()).into_tuple();

                let mut adjusted_tokens = tokens;
                for t in &mut adjusted_tokens {
                    t.span.start += body_offset;
                    t.span.end += body_offset;
                }

                for s in &mut scopes {
                    s.span.start += body_offset;
                    s.span.end += body_offset;
                }

                for e in &mut errors {
                    let span = match e {
                        ScopenizeError::UnclosedInlineNote(s) => s,
                        ScopenizeError::BackRefFailed(s) => s,
                        ScopenizeError::InvalidRubyDelimiterUsage(s) => s,
                        ScopenizeError::CrossingNote(s) => s,
                        ScopenizeError::IsolatedEndNote(s) => s,
                    };
                    span.start += body_offset;
                    span.end += body_offset;
                }

                ParsedData {
                    meta,
                    body_offset,
                    symbol_block,
                    tokens: adjusted_tokens,
                    scopes,
                    errors,
                }
            },
        }
        .build();

        Self { inner, line_index }
    }

    pub fn text(&self) -> &str {
        self.inner.borrow_text()
    }

    pub fn parsed(&self) -> &ParsedData<'_> {
        self.inner.borrow_parsed()
    }

    pub fn token_at_offset(&self, offset: usize) -> Option<&Tokenized<'_>> {
        self.parsed()
            .tokens
            .iter()
            .find(|t| t.span.start <= offset && offset < t.span.end)
    }

    pub fn scopes_at_offset(&self, offset: usize) -> Vec<&Scope<'_>> {
        self.parsed()
            .scopes
            .iter()
            .filter(|s| s.span.start <= offset && offset < s.span.end)
            .collect()
    }

    pub fn offset_at_position(&self, pos: Position) -> usize {
        self.line_index.position_to_offset(self.text(), pos)
    }
}

