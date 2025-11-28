//! XDL Language Server Protocol (LSP) implementation
//!
//! This binary provides language intelligence features for XDL files
//! including diagnostics, completion, hover, go-to-definition, and more.

use tower_lsp::{LspService, Server};
use tracing_subscriber::prelude::*;

mod completion;
mod diagnostics;
mod document;
mod goto;
mod hover;
mod semantic_tokens;
mod server;
mod symbols;
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

    let (service, socket) = LspService::new(server::XdlLanguageServer::new);

    Server::new(stdin, stdout, socket).serve(service).await;
}
