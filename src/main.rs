mod method;
use lsp_types::request::Request;

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

                handle_request(&mut backend, &connection, req)?;
            }

            lsp_server::Message::Response(_rsp) => {}

            lsp_server::Message::Notification(_nfy) => {}
        }
    }

    Ok(())
}

fn handle_request(
    rt: &mut method::LspBackend,
    conn: &lsp_server::Connection,
    req: lsp_server::Request,
) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
    let id = req.id.clone();
    let mut rsp = match req.method.as_str() {
        lsp_types::request::GotoDefinition::METHOD => {
            let p = serde_json::from_value(req.params)?;
            method::goto_definition::goto_definition(rt, p)?
        },

        // Method not found.
        _ => {
            lsp_server::Response {
                id: 0.into(),
                result: None,
                error: Some(lsp_server::ResponseError {
                    code: lsp_server::ErrorCode::MethodNotFound as i32,
                    message: format!("method not found: {}", req.method),
                    data: None,
                }),
            }
        }
    };

    rsp.id = id;
    conn.sender.send(lsp_server::Message::Response(rsp))?;

    Ok(())
}
