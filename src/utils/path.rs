/// Get all items in the given path.
///
/// # Arguments
///
/// + `path` - Path to project root.
///
/// # Returns
///
/// + List of entry.
pub fn walk_with_gitignore(path: std::path::PathBuf) -> std::io::Result<Vec<std::path::PathBuf>> {
    let mut files_info = Vec::new();
    visit_dirs(path.as_path(), &mut files_info)?;
    Ok(files_info)
}

fn visit_dirs(
    dir: &std::path::Path,
    files_info: &mut Vec<std::path::PathBuf>,
) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, files_info)?;
            } else {
                files_info.push(path);
            }
        }
    }
    Ok(())
}
