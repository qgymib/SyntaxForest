#[derive(Debug, Default, Clone)]
pub struct FileInfo {
    /// The path of the file.
    pub path: std::path::PathBuf,

    /// The modify time of the file.
    pub mtime: i64,

    /// The parser time of the file.
    pub ptime: i64,
}

/// Get all items in the given path.
///
/// # Arguments
///
/// + `path` - Path to project root.
///
/// # Returns
///
/// + List of entry.
pub fn walk_with_gitignore(path: std::path::PathBuf) -> std::io::Result<Vec<FileInfo>> {
    let mut files_info = Vec::new();

    for ret in ignore::Walk::new(path) {
        match ret {
            Ok(e) => {
                let path = e.path().to_path_buf();
                let metadata = e.metadata().unwrap();
                let mtime = metadata.modified()?;
                let mtime = mtime.duration_since(std::time::UNIX_EPOCH).unwrap();
                let mtime = mtime.as_secs();

                if metadata.is_file() {
                    files_info.push(FileInfo {
                        path: path,
                        mtime: mtime as i64,
                        ..Default::default()
                    });
                }
            }
            Err(e) => tracing::error!("ERROR: {}", e),
        }
    }

    Ok(files_info)
}
