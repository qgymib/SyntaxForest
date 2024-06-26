mod common;

#[test]
fn c_parser() {
    let cargo_target_tmpdir = env!("CARGO_TARGET_TMPDIR");
    let dbfile_path = format!("{}/tags.db", cargo_target_tmpdir);

    common::asset::Asset::cleanup_and_extract(cargo_target_tmpdir).unwrap();

    let mut client = common::lsp_client::LspClient::new().unwrap();
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
        logdir: Some(cargo_target_tmpdir.to_string()),
        loglevel: Some("TRACE".into()),
        ..Default::default()
    };
    syntax_forest::start_lsp(&config).unwrap();

    thread_handle.join().unwrap();
    client.close().unwrap();
}
