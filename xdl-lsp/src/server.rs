//! LSP Server implementation using tower-lsp

use dashmap::DashMap;
use std::sync::Arc;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::document::DocumentState;
use crate::semantic_tokens::semantic_tokens_legend;
use crate::symbols::SymbolTable;
use crate::{completion, goto, hover, symbols};

pub struct XdlLanguageServer {
    client: Client,
    documents: Arc<DashMap<Url, DocumentState>>,
    symbol_table: Arc<tokio::sync::RwLock<SymbolTable>>,
}

impl XdlLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(DashMap::new()),
            symbol_table: Arc::new(tokio::sync::RwLock::new(SymbolTable::new())),
        }
    }

    async fn on_change(&self, uri: Url, text: String, version: i32) {
        // Parse the document and update state
        let doc_state = DocumentState::parse(text, version);

        // Publish diagnostics
        let diagnostics = doc_state.diagnostics.clone();
        self.documents.insert(uri.clone(), doc_state);

        self.client
            .publish_diagnostics(uri, diagnostics, Some(version))
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for XdlLanguageServer {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(true),
                        })),
                        ..Default::default()
                    },
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![
                        ".".to_string(),
                        "!".to_string(),
                        ",".to_string(),
                    ]),
                    resolve_provider: Some(false),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: semantic_tokens_legend(),
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            range: Some(true),
                            ..Default::default()
                        },
                    ),
                ),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "xdl-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        tracing::info!("XDL Language Server initialized");
    }

    async fn shutdown(&self) -> Result<()> {
        tracing::info!("XDL Language Server shutting down");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        tracing::debug!("Document opened: {}", params.text_document.uri);
        self.on_change(
            params.text_document.uri,
            params.text_document.text,
            params.text_document.version,
        )
        .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.into_iter().next() {
            self.on_change(
                params.text_document.uri,
                change.text,
                params.text_document.version,
            )
            .await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        if let Some(text) = params.text {
            self.on_change(params.text_document.uri, text, 0).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        tracing::debug!("Document closed: {}", params.text_document.uri);
        self.documents.remove(&params.text_document.uri);
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        if let Some(doc) = self.documents.get(uri) {
            let symbol_table = self.symbol_table.read().await;
            Ok(completion::provide_completions(
                &doc,
                position,
                &symbol_table,
            ))
        } else {
            Ok(None)
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some(doc) = self.documents.get(uri) {
            let symbol_table = self.symbol_table.read().await;
            Ok(hover::provide_hover(&doc, position, &symbol_table))
        } else {
            Ok(None)
        }
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some(doc) = self.documents.get(uri) {
            Ok(goto::goto_definition(&doc, position, uri))
        } else {
            Ok(None)
        }
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        if let Some(doc) = self.documents.get(uri) {
            Ok(goto::find_references(&doc, position, uri))
        } else {
            Ok(None)
        }
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = &params.text_document.uri;

        if let Some(doc) = self.documents.get(uri) {
            Ok(Some(DocumentSymbolResponse::Nested(
                symbols::get_document_symbols(&doc),
            )))
        } else {
            Ok(None)
        }
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = &params.text_document.uri;

        if let Some(doc) = self.documents.get(uri) {
            Ok(crate::semantic_tokens::compute_semantic_tokens(&doc))
        } else {
            Ok(None)
        }
    }
}
