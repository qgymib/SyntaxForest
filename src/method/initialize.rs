use lsp_types::*;

use super::LspBackend;

pub fn initialize(
    rt: &mut LspBackend,
    conn: &lsp_server::Connection,
) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
    let server_capabilities = serde_json::to_value(get_server_capacity()).unwrap();
    let initialization_params = match conn.initialize(server_capabilities) {
        Ok(it) => it,
        Err(e) => {
            return Err(e.into());
        }
    };

    // Parse the initialization parameters.
    let initialization_params: lsp_types::InitializeParams =
        serde_json::from_value(initialization_params)?;
    copy_workspace_folder(rt, &initialization_params);

    // Create the database.
    let conn = rusqlite::Connection::open("syntax_forest.db").unwrap();
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
    )
    .unwrap();

    // Assign the database to the runtime.
    rt.db = Some(std::sync::Arc::new(std::sync::Mutex::new(conn)));

    for folder in &rt.workspace_folders {
        let file_path = folder.uri.to_file_path().unwrap();
        let file_list = crate::utils::path::walk_with_gitignore(file_path)?;
        for path in file_list {
            let db = rt.db.as_ref().unwrap().lock().unwrap();
            db.execute(
                "INSERT INTO files (path, mtime, ptime) VALUES (?1, ?2, 0)",
                (&path.path.to_str(), &path.mtime),
            )
            .unwrap();
        }
    }

    trigger_tree_sitter(rt);

    Ok(())
}

fn trigger_tree_sitter(rt: &mut LspBackend) {
    let files = find_files_need_to_parser(rt).unwrap();

    for file in files {
        do_tree_sitter(rt, &file);
    }
}

fn find_files_need_to_parser(
    rt: &mut LspBackend,
) -> Result<Vec<crate::utils::path::FileInfo>, Box<dyn std::error::Error + Sync + Send>> {
    let mut ret = Vec::new();

    let db = rt.db.as_ref().unwrap().lock().unwrap();

    let cur_time = std::time::SystemTime::now();
    let cur_time = cur_time.duration_since(std::time::UNIX_EPOCH).unwrap();
    let cur_time = cur_time.as_secs() as i64;
    let query_stmt = format!("SELECT * FROM files WHERE ptime < {}", cur_time);
    let mut stmt = db.prepare(query_stmt.as_str()).unwrap();
    let file_iter = stmt
        .query_map([], |row| {
            let path: String = row.get(0)?;
            Ok(crate::utils::path::FileInfo {
                path: path.into(),
                mtime: row.get(1)?,
                ptime: row.get(2)?,
            })
        })
        .unwrap();

    for file in file_iter {
        ret.push(file.unwrap());
    }

    Ok(ret)
}

fn do_tree_sitter(_rt: &mut LspBackend, _file: &crate::utils::path::FileInfo) {}

/// Safe workspace folders from client initialize params.
///
/// # Arguments
///
/// + `dst` - A mut reference to Runtime.
/// + `src` - Reference to InitializeParams
fn copy_workspace_folder(dst: &mut LspBackend, src: &InitializeParams) {
    match &src.root_uri {
        Some(value) => dst.workspace_folders.push(WorkspaceFolder {
            name: String::from(""),
            uri: value.clone(),
        }),
        None => (),
    };

    match &src.workspace_folders {
        Some(value) => dst.workspace_folders = value.clone(),
        None => (),
    };
}

/// Get the default server capabilities.
///
/// Returns
///
/// + `ServerCapabilities` - The default server capabilities.
fn get_server_capacity() -> ServerCapabilities {
    return ServerCapabilities {
        position_encoding: Some(PositionEncodingKind::UTF8),
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        completion_provider: Some(CompletionOptions {
            ..Default::default()
        }),
        definition_provider: Some(OneOf::Right(DefinitionOptions {
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: Some(true),
            },
        })),
        type_definition_provider: Some(TypeDefinitionProviderCapability::Simple(true)),
        implementation_provider: Some(ImplementationProviderCapability::Simple(true)),
        references_provider: Some(OneOf::Right(ReferencesOptions {
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: Some(true),
            },
        })),
        document_symbol_provider: Some(OneOf::Left(true)),
        workspace_symbol_provider: Some(OneOf::Right(WorkspaceSymbolOptions {
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: Some(true),
            },
            resolve_provider: Some(false),
        })),
        declaration_provider: Some(DeclarationCapability::Simple(true)),
        workspace: Some(WorkspaceServerCapabilities {
            workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                supported: Some(true),
                change_notifications: Some(OneOf::Left(false)),
            }),
            file_operations: Some(WorkspaceFileOperationsServerCapabilities {
                did_rename: Some(FileOperationRegistrationOptions {
                    filters: vec![FileOperationFilter {
                        scheme: Option::None,
                        pattern: FileOperationPattern {
                            glob: String::from("*"),
                            ..Default::default()
                        },
                    }],
                }),
                did_delete: Some(FileOperationRegistrationOptions {
                    filters: vec![FileOperationFilter {
                        scheme: Option::None,
                        pattern: FileOperationPattern {
                            glob: String::from("*"),
                            ..Default::default()
                        },
                    }],
                }),
                ..Default::default()
            }),
        }),
        ..ServerCapabilities::default()
    };
}
