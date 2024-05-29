use clap::Parser;

fn main() -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
    // Parse the command line arguments.
    let config = syntax_forest::LspConfig::parse();

    // Start the LSP server.
    syntax_forest::start_lsp(&config)?;

    // Return success.
    Ok(())
}
