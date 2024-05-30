mod common;

#[test]
fn c_parser() {
    let cargo_target_tmpdir = env!("CARGO_TARGET_TMPDIR");
    let dbfile_path = format!("{}/tags.db", cargo_target_tmpdir);
    let _ = std::fs::remove_file(&dbfile_path);

    let mut client = common::LspClient::new().unwrap();
    let port = client.local_addr().port();

    let mut client_copy = client.clone();
    let thread_handle = std::thread::spawn(move || {
        client_copy.initialize().unwrap();
        client_copy.shutdown().unwrap();
    });

    // Start lsp server.
    let config = syntax_forest::LspConfig {
        dbfile: Some(dbfile_path),
        port: Some(port),
        ..Default::default()
    };
    syntax_forest::start_lsp(&config).unwrap();

    thread_handle.join().unwrap();
    client.close().unwrap();
}
