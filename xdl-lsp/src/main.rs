//! XDL Language Server Protocol (LSP) implementation
//!
//! This binary provides language intelligence features for XDL files
//! including diagnostics, completion, hover, go-to-definition, and more.

use tower_lsp::{LspService, Server};
use tracing_subscriber::prelude::*;

mod server;
mod document;
mod diagnostics;
mod symbols;
mod completion;
mod hover;
mod goto;
mod semantic_tokens;
mod utils;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_ansi(false),
        )
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting XDL Language Server");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| server::XdlLanguageServer::new(client));

    Server::new(stdin, stdout, socket).serve(service).await;
}
