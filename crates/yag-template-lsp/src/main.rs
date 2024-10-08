use std::{env, io};

use anyhow::Context;
use server::YagTemplateLanguageServer;
use tower_lsp::{LspService, Server};
use tracing_subscriber::filter::Targets;
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{Layer, Registry};

mod provider;
mod server;
mod session;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::build(YagTemplateLanguageServer::new).finish();

    setup_logging()?;
    Server::new(stdin, stdout, socket).serve(service).await;
    Ok(())
}

fn setup_logging() -> anyhow::Result<()> {
    let filter = get_log_filter()?;

    let stderr_writer = BoxMakeWriter::new(io::stderr);
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_writer(stderr_writer)
        .with_ansi(false)
        .with_filter(filter);
    Registry::default().with(fmt_layer).init();
    Ok(())
}

fn get_log_filter() -> anyhow::Result<Targets> {
    /// Make tower_lsp less noisy, but otherwise show info logs by default.
    const DEFAULT_LOG_FILTER: &str = "tower_lsp=error,info";

    let filter = env::var("YAG_LSP_LOG")
        .ok()
        .unwrap_or_else(|| DEFAULT_LOG_FILTER.into());
    filter
        .parse()
        .with_context(|| format!("invalid log filter: `{filter}`"))
}
