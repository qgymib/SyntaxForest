use super::LspBackend;

pub fn shutdown(_rt: &mut LspBackend) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
    Ok(())
}
