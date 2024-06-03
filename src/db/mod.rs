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
    pub fn new(conn: rusqlite::Connection) -> rusqlite::Result<SqliteClient> {
        tracing::debug!("sqlite version: {}", rusqlite::version());

        let client = SqliteClient {
            conn: std::sync::Arc::new(std::sync::Mutex::new(conn)),
        };

        client.initialize_tables()?;

        return Ok(client);
    }

    pub fn startup_scan(&self, files: &Vec<FileInfo>) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();

        // The `startup_scan_files` table is used to store the files scanned during startup.
        conn.execute(
            "CREATE TABLE startup_scan_files (
                path TEXT PRIMARY KEY NOT NULL,
                mtime INTEGER
            );",
            (),
        )?;

        // Insert files into temporary table.
        for file in files {
            conn.execute(
                "INSERT INTO startup_scan_files (path, mtime) VALUES (?1, ?2);",
                (&file.path.to_str(), file.mtime),
            )?;
        }

        self.remove_non_exist_records(&conn)?;
        self.update_mtime(&conn)?;

        // Drop the temopry tables.
        //conn.execute("DROP TABLE startup_scan_files;", ())?;

        Ok(())
    }

    pub fn pending_analysis(&self) -> rusqlite::Result<Vec<FileInfo>> {
        let mut ret = Vec::new();
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare("SELECT * FROM files WHERE ptime < mtime")?;
        let iter = stmt.query_map([], |row| {
            let path: String = row.get(0)?;
            Ok(FileInfo {
                path: path.into(),
                mtime: row.get(1)?,
                ptime: row.get(2)?,
            })
        })?;

        for file in iter {
            ret.push(file.unwrap());
        }

        return Ok(ret);
    }

    fn update_mtime(
        &self,
        conn: &std::sync::MutexGuard<rusqlite::Connection>,
    ) -> rusqlite::Result<()> {
        conn.execute(
            "INSERT INTO files (path, mtime, ptime)
            SELECT path, mtime, 0
            FROM startup_scan_files
            WHERE true
            ON CONFLICT(path) DO UPDATE SET
                mtime = EXCLUDED.mtime;",
            (),
        )
        .unwrap();

        Ok(())
    }

    fn remove_non_exist_records(
        &self,
        conn: &std::sync::MutexGuard<rusqlite::Connection>,
    ) -> rusqlite::Result<()> {
        // Create table `files_to_delete`.
        conn.execute(
            "CREATE TABLE files_to_delete (
                path TEXT PRIMARY KEY NOT NULL
            );",
            (),
        )?;

        // Find all files that do not exist in the filesystem.
        conn.execute(
            "INSERT INTO files_to_delete (path)
            SELECT f.path
            FROM files f
            WHERE NOT EXISTS (
                SELECT 1
                FROM startup_scan_files tf
                WHERE tf.path = f.path
            );",
            (),
        )?;

        //conn.execute("DROP TABLE files_to_delete;", ())?;

        Ok(())
    }

    /// Setup required tables.
    fn initialize_tables(&self) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();

        // The `files`` table is used to store the file information.
        conn.execute(
            "CREATE TABLE IF NOT EXISTS files (
                path TEXT PRIMARY KEY NOT NULL,
                mtime INTEGER,
                ptime INTEGER
            )",
            (),
        )?;

        // The `tags` table is used to store the tag information.
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                type INTEGER,
                beg_row INTEGER,
                beg_col INTEGER,
                end_row INTEGER,
                end_col INTEGER,
                path TEXT,
                name TEXT,
                FOREIGN KEY(path) REFERENCES files(path)
            )",
            (),
        )?;

        // The `xrefs` table is used to store the xref information.
        conn.execute(
            "CREATE TABLE IF NOT EXISTS xrefs (
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
