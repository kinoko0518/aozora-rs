mod completion;
mod document;
mod folding;
mod hover;
mod line_index;
mod semantic_tokens;
mod server;

use dashmap::DashMap;
use tower_lsp::{LspService, Server};

use server::AozoraLsp;

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| AozoraLsp {
        client,
        documents: DashMap::new(),
    });

    Server::new(stdin, stdout, socket).serve(service).await;
}
