mod method;

fn main() -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
    const PROG_NAME: &str = env!("CARGO_PKG_NAME");
    const PROG_VERSION: &str = env!("CARGO_PKG_VERSION");

    let mut backend = method::LspBackend {
        prog_name: PROG_NAME.to_string(),
        prog_version: PROG_VERSION.to_string(),
        ..Default::default()
    };

    // Initialize logging
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();
    tracing::info!("{} v{}", PROG_NAME, PROG_VERSION);
    tracing::info!("PID: {}", std::process::id());

    // Create the transport. Includes the stdio (stdin and stdout) versions but this could
    // also be implemented to use sockets or HTTP.
    let (connection, io_threads) = lsp_server::Connection::stdio();

    // Initialize the server.
    match method::initialize::initialize(&mut backend, &connection) {
        Ok(_) => {}
        Err(e) => {
            io_threads.join()?;
            return Err(e.into());
        }
    }

    main_loop(backend, connection)?;
    io_threads.join()?;

    Ok(())
}

fn main_loop(
    mut backend: method::LspBackend,
    connection: lsp_server::Connection,
) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
    for msg in &connection.receiver {
        match msg {
            lsp_server::Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    method::shutdown::shutdown(&mut backend)?;
                    return Ok(());
                }
            }

            lsp_server::Message::Response(rsp) => {}

            lsp_server::Message::Notification(nfy) => {}
        }
    }

    Ok(())
}
