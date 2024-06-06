use server::YagTemplateLanguageServer;
use tower_lsp::{LspService, Server};

mod diagnostics;
mod mapper;
mod server;
mod workspace;

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::build(YagTemplateLanguageServer::new).finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
