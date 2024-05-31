#[derive(Debug, Default, Clone)]
pub struct FileInfo {
    /// The path of the file.
    pub path: std::path::PathBuf,

    /// The modify time of the file.
    pub mtime: i64,

    /// The parser time of the file.
    pub ptime: i64,
}

/// Sqlite database implementation
#[derive(Debug, Clone)]
pub struct SqliteClient {
    conn: std::sync::Arc<std::sync::Mutex<rusqlite::Connection>>,
}

impl SqliteClient {
    /// Create a new Sqlite connection client.
    ///
    /// # Arguments
    ///
    /// + `conn` - The database connection.
    pub fn new(conn: rusqlite::Connection) -> SqliteClient {
        SqliteClient {
            conn: std::sync::Arc::new(std::sync::Mutex::new(conn)),
        }
    }

    /// Setup required tables.
    pub fn initialize_tables(&self) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS files (
                path TEXT PRIMARY KEY,
                mtime INTEGER,
                ptime INTEGER
            );
            CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                type INTEGER,
                beg_row INTEGER,
                beg_col INTEGER,
                end_row INTEGER,
                end_col INTEGER,
                file TEXT,
                name TEXT,
                FOREIGN KEY(file) REFERENCES files(path)
            );
            CREATE TABLE IF NOT EXISTS xrefs (
                name INTEGER,
                hold INTEGER,
                FOREIGN KEY(name) REFERENCES tags(id),
                FOREIGN KEY(hold) REFERENCES tags(id)
            )",
            (),
        )?;
        Ok(())
    }
}
