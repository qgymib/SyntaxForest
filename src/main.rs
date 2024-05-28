mod method;

#[derive(Debug)]
struct LspRuntime {
    /// The name of the program.
    prog_name: String,

    /// The version of the program.
    prog_version: String,

    /// The list of workspace folders.
    workspace_folders: Vec<tower_lsp::lsp_types::WorkspaceFolder>,

    /// The database.
    db: rusqlite::Connection,
}

#[derive(Debug)]
pub struct LspBackend {
    client: tower_lsp::Client,
    runtime: tokio::sync::Mutex<LspRuntime>,
}

#[tower_lsp::async_trait]
impl tower_lsp::LanguageServer for LspBackend {
    async fn initialize(
        &self,
        params: tower_lsp::lsp_types::InitializeParams,
    ) -> tower_lsp::jsonrpc::Result<tower_lsp::lsp_types::InitializeResult> {
        return method::initialize::initialize(self, params).await;
    }

    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    const PROG_NAME: &str = env!("CARGO_PKG_NAME");
    const PROG_VERSION: &str = env!("CARGO_PKG_VERSION");

    // Initialize logging
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();
    tracing::info!("{} v{}", PROG_NAME, PROG_VERSION);
    tracing::info!("PID: {}", std::process::id());

    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS files (
            path TEXT PRIMARY KEY,
            mtime INTEGER,
            ptime INTEGER,
        );
        CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            type INTEGER,
            beg_row INTEGER,
            beg_col INTEGER,
            end_row INTEGER,
            end_col INTEGER,
            FOREIGN KEY(file) REFERENCES files(path),
            name TEXT,
        );
        CREATE TABLE IF NOT EXISTS xrefs (
            FOREIGN KEY(name) REFERENCES tags(id),
            FOREIGN KEY(hold) REFERENCES tags(id),
        )",
        (),
    )
    .unwrap();

    // Generate the LSP service.
    let runtime = tokio::sync::Mutex::new(LspRuntime {
        prog_name: PROG_NAME.to_string(),
        prog_version: PROG_VERSION.to_string(),
        workspace_folders: Vec::new(),
        db: conn,
    });
    let (service, socket) = tower_lsp::LspService::new(|client| LspBackend {
        client: client,
        runtime: runtime,
    });

    // Start the LSP server on stdio.
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    tower_lsp::Server::new(stdin, stdout, socket)
        .serve(service)
        .await;
}
