#[derive(rust_embed::Embed)]
#[folder = "tests/sample"]
pub struct Asset;

impl Asset {
    /// Extract all assets to the given path.
    ///
    /// # Arguments
    ///
    /// + `path` - Path to extract.
    pub fn extract_all(path: &str) -> std::io::Result<()> {
        for item in Asset::iter() {
            let name = item.as_ref();

            let data = Asset::get(name).unwrap();
            let content = data.data.as_ref();

            let full_path = format!("{}/{}", path, name);
            std::fs::write(full_path, content)?;
        }

        Ok(())
    }

    /// Cleanup all items in the given path.
    ///
    /// # Arguments
    ///
    /// + `path` - Path to cleanup.
    pub fn cleanup_all(path: &str) -> std::io::Result<()> {
        // Read the directory contents
        let entries = std::fs::read_dir(path)?;

        // Iterate over each entry and delete all items.
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                std::fs::remove_dir_all(path)?;
            } else {
                std::fs::remove_file(path)?;
            }
        }

        Ok(())
    }

    /// Cleanup and extract all assets to the given path.
    ///
    /// # Arguments
    ///
    /// + `path` - Path to extract.
    pub fn cleanup_and_extract(path: &str) -> std::io::Result<()> {
        Self::cleanup_all(path)?;
        Self::extract_all(path)?;
        Ok(())
    }
}
