use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::completion::compute_completions;
use crate::document::DocumentState;
use crate::folding::compute_folding_ranges;
use crate::hover::compute_hover;
use crate::semantic_tokens::{self, compute_semantic_tokens};

pub struct AozoraLsp {
    pub client: Client,
    pub documents: DashMap<Url, DocumentState>,
}

#[tower_lsp::async_trait]
impl LanguageServer for AozoraLsp {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: semantic_tokens::legend(),
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            range: None,
                            ..Default::default()
                        },
                    ),
                ),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec!["＃".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "aozora-lsp initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        self.reparse(uri, text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        // TextDocumentSyncKind::FULLなので全文が1つ目のイベントに入る
        if let Some(change) = params.content_changes.into_iter().next() {
            // 登録済みのドキュメントの場合のみリパース
            if self.documents.contains_key(&uri) {
                self.reparse(uri, change.text).await;
            } else {
                // 未登録なら新規判定を試みる
                self.reparse(uri, change.text).await;
            }
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.remove(&params.text_document.uri);
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = &params.text_document.uri;
        let Some(doc) = self.documents.get(uri) else {
            return Ok(None);
        };
        let tokens = compute_semantic_tokens(&doc);
        Ok(Some(SemanticTokensResult::Tokens(tokens)))
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;
        let Some(doc) = self.documents.get(uri) else {
            return Ok(None);
        };

        // カーソル位置から後方スキャンして直近の `［＃` を探す
        let offset = doc.offset_at_position(pos);
        let text_before = &doc.text[..offset];

        let trigger = "［＃";
        let Some(trigger_pos) = text_before.rfind(trigger) else {
            return Ok(None);
        };

        // `［＃`の直後から現在位置までがプレフィックス
        let after_trigger = trigger_pos + trigger.len();
        // `］`が間にあれば既に閉じた注記なのでスキップ
        let between = &doc.text[after_trigger..offset];
        if between.contains('］') {
            return Ok(None);
        }

        let prefix = between;
        let replace_start = doc.line_index.offset_to_position(&doc.text, after_trigger);
        let replace_end = pos;
        let replace_range = Range {
            start: replace_start,
            end: replace_end,
        };

        let items = compute_completions(&doc, prefix, replace_range);
        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;
        let Some(doc) = self.documents.get(uri) else {
            return Ok(None);
        };
        Ok(compute_hover(&doc, pos))
    }

    async fn folding_range(&self, params: FoldingRangeParams) -> Result<Option<Vec<FoldingRange>>> {
        let uri = &params.text_document.uri;
        let Some(doc) = self.documents.get(uri) else {
            return Ok(None);
        };
        let ranges = compute_folding_ranges(&doc);
        Ok(Some(ranges))
    }
}

impl AozoraLsp {
    async fn reparse(&self, uri: Url, text: String) {
        match DocumentState::parse(text) {
            Some(state) => {
                self.documents.insert(uri, state);
            }
            None => {
                // メタデータ解析失敗 → 青空文庫書式ではないので無視
                self.documents.remove(&uri);
            }
        }
    }
}
