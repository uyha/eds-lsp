use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use eds_lsp::utils::ts_to_lsp_range;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::OneOf::*;
use tower_lsp::lsp_types::Url;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use eds_lsp::eds::*;

struct Backend {
    client: Client,
    trees: Arc<Mutex<HashMap<Url, EDS>>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                document_symbol_provider: Some(Left(true)),
                inlay_hint_provider: Some(Left(true)),
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let map = &mut self.trees.lock().unwrap();
        let eds = &map.get(&params.text_document.uri).unwrap();
        let symbols = eds
            .into_iter()
            .map(|node| DocumentSymbol {
                name: node.node.kind().to_owned(),
                detail: None,
                kind: SymbolKind::OPERATOR,
                tags: None,
                deprecated: None,
                range: ts_to_lsp_range(node.node.range()),
                selection_range: ts_to_lsp_range(node.node.range()),
                children: None,
            })
            .collect();

        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let map = &mut self.trees.lock().unwrap();
        let uri = &params.text_document.uri;
        let content = &params.text_document.text;

        if let Some(eds) = EDS::parse(content) {
            map.insert(uri.clone(), eds);
        }
    }

    async fn inline_value(&self, params: InlineValueParams) -> Result<Option<Vec<InlineValue>>> {
        let map = &mut self.trees.lock().unwrap();
        let eds = &map.get(&params.text_document.uri).unwrap();

        Ok(Some(vec![InlineValue::Text(InlineValueText {
            range: Range {
                start: Position {
                    line: params.range.start.line,
                    character: params.range.start.character,
                },
                end: Position {
                    line: params.range.end.line,
                    character: params.range.end.character,
                },
            },
            text: "asdfasdf".to_owned(),
        })]))
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        trees: Arc::new(Mutex::new(HashMap::<Url, EDS>::new())),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
