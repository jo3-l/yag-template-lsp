use std::io;

use server::YagTemplateLanguageServer;
use tower_lsp::{LspService, Server};
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Registry;

mod provider;
mod server;
mod session;

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::build(YagTemplateLanguageServer::new).finish();

    setup_logging();
    Server::new(stdin, stdout, socket).serve(service).await;
}

fn setup_logging() {
    let stderr_writer = BoxMakeWriter::new(io::stderr);
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_writer(stderr_writer)
        .with_ansi(false);
    Registry::default().with(fmt_layer).init();
}
