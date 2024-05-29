pub fn shutdown(
    _rt: &mut crate::LspRuntime,
) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
    Ok(())
}
