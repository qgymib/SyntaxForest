mod method;
mod utils;

use lsp_types::request::Request;

#[derive(Debug, clap::Parser, Default, Clone)]
#[command(author, version, about, long_about = None)]
pub struct LspConfig {
    #[arg(
        long,
        conflicts_with = "port",
        help = "Uses stdio as the communication channel"
    )]
    pub stdio: bool,

    #[arg(
        long,
        conflicts_with = "stdio",
        help = "Uses a socket as the communication channel",
        long_help = "The LSP server start as TCP client and connect to the specified port."
    )]
    pub port: Option<u16>,

    #[arg(
        long,
        value_name = "DIR",
        help = "Specifies a directory to use for logging"
    )]
    pub logdir: Option<String>,

    #[arg(
        long,
        value_name = "STRING",
        help = "Set log leve",
        long_help = "Possible values are: [OFF | TRACE | DEBUG | INFO | WARN | ERROR]. By default `INFO` is used."
    )]
    pub loglevel: Option<String>,
}

pub fn start_lsp(config: &LspConfig) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
    const PROG_NAME: &str = env!("CARGO_PKG_NAME");
    const PROG_VERSION: &str = env!("CARGO_PKG_VERSION");

    // Setup logging system.
    setup_logging_system(config, PROG_NAME);
    tracing::info!("{} - v{}", PROG_NAME, PROG_VERSION);
    tracing::info!("PID: {}", std::process::id());

    // Create the transport.
    let (connection, io_threads) = if config.stdio {
        lsp_server::Connection::stdio()
    } else {
        let port = config.port.expect("port is required");
        lsp_server::Connection::connect(format!("127.0.0.1:{}", port)).unwrap()
    };

    // Initialize the server.
    tracing::info!("initialize...");

    let mut backend = method::LspBackend {
        prog_name: PROG_NAME.to_string(),
        prog_version: PROG_VERSION.to_string(),
        ..Default::default()
    };
    match method::initialize::initialize(&mut backend, &connection) {
        Ok(_) => {}
        Err(e) => {
            io_threads.join()?;
            return Err(e.into());
        }
    }

    tracing::info!("starting lsp");
    message_loop(backend, connection)?;
    io_threads.join()?;

    Ok(())
}

fn setup_logging_system(config: &LspConfig, prog_name: &str) {
    // Get log level.
    let loglevel = match &config.loglevel {
        Some(v) => v.clone(),
        None => String::from("INFO"),
    };

    // Parse log level.
    let loglevel = match loglevel.to_lowercase().as_str() {
        "off" => tracing::metadata::LevelFilter::OFF,
        "trace" => tracing::metadata::LevelFilter::TRACE,
        "debug" => tracing::metadata::LevelFilter::DEBUG,
        "info" => tracing::metadata::LevelFilter::INFO,
        "warn" => tracing::metadata::LevelFilter::WARN,
        "error" => tracing::metadata::LevelFilter::ERROR,
        unmatched => panic!(
            "Parser command line argument failed: unknown option value `{}`",
            unmatched
        ),
    };

    match &config.logdir {
        Some(path) => {
            let logfile = format!("{}.log", prog_name);
            let file_appender = tracing_appender::rolling::never(path, logfile);
            tracing_subscriber::fmt()
                .with_max_level(loglevel)
                .with_writer(file_appender)
                .with_ansi(false)
                .init();
        }
        None => {
            tracing_subscriber::fmt()
                .with_max_level(loglevel)
                .with_writer(std::io::stderr)
                .init();
        }
    }
    std::panic::set_hook(Box::new(tracing_panic::panic_hook));
}

fn message_loop(
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
        }

        // Method not found.
        _ => lsp_server::Response {
            id: 0.into(),
            result: None,
            error: Some(lsp_server::ResponseError {
                code: lsp_server::ErrorCode::MethodNotFound as i32,
                message: format!("method not found: {}", req.method),
                data: None,
            }),
        },
    };

    rsp.id = id;
    conn.sender.send(lsp_server::Message::Response(rsp))?;

    Ok(())
}
