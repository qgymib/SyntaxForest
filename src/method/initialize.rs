use lsp_types::*;

use crate::LspRuntime;

pub fn initialize(
    conn: &lsp_server::Connection,
    config: &crate::LspConfig,
) -> Result<crate::LspRuntime, Box<dyn std::error::Error + Sync + Send>> {
    let server_capabilities = serde_json::to_value(get_server_capacity()).unwrap();
    let initialization_params = match conn.initialize(server_capabilities) {
        Ok(it) => it,
        Err(e) => {
            return Err(e.into());
        }
    };

    // Create the database.
    let conn = match &config.dbfile {
        Some(path) => rusqlite::Connection::open(path).unwrap(),
        None => rusqlite::Connection::open_in_memory().unwrap(),
    };
    let client = crate::db::SqliteClient::new(conn);

    let mut rt = LspRuntime {
        workspace_folders: vec![],
        db: client,
        file_association_table: std::collections::BTreeMap::new(),
    };

    // Parse the initialization parameters.
    let initialization_params: lsp_types::InitializeParams =
        serde_json::from_value(initialization_params)?;
    copy_workspace_folder(&mut rt, &initialization_params);

    rt.db.initialize_tables();

    for folder in &rt.workspace_folders {
        let file_path = folder.uri.to_file_path().unwrap();
        let file_list = crate::utils::path::walk_with_gitignore(file_path)?;
        for path in file_list {
            rt.db
                .execute(
                    "INSERT INTO files (path, mtime, ptime) VALUES (?1, ?2, 0)",
                    (&path.path.to_str(), &path.mtime),
                )
                .unwrap();
        }
    }

    rt.file_association_table
        .insert(String::from(".c"), tree_sitter_c::language());
    rt.file_association_table
        .insert(String::from(".h"), tree_sitter_c::language());

    trigger_tree_sitter(&mut rt);

    Ok(rt)
}

fn trigger_tree_sitter(rt: &mut crate::LspRuntime) {
    let files = find_files_need_to_parser(rt).unwrap();

    for file in files {
        do_tree_sitter(rt, &file);
    }
}

fn find_files_need_to_parser(
    rt: &mut crate::LspRuntime,
) -> Result<Vec<crate::utils::path::FileInfo>, Box<dyn std::error::Error + Sync + Send>> {
    let mut ret = Vec::new();

    let cur_time = std::time::SystemTime::now();
    let cur_time = cur_time.duration_since(std::time::UNIX_EPOCH).unwrap();
    let cur_time = cur_time.as_secs() as i64;
    let query_stmt = format!("SELECT * FROM files WHERE ptime < {}", cur_time);
    let mut stmt = rt.db.prepare(query_stmt.as_str()).unwrap();
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

fn do_tree_sitter(rt: &mut crate::LspRuntime, file: &crate::utils::path::FileInfo) {
    let path = file.path.to_str().unwrap();

    for (k, v) in &rt.file_association_table {
        if path.ends_with(k.as_str()) {
            do_tree_sitter_language(rt, &file.path, v);
            return;
        }
    }
}

fn do_tree_sitter_language(
    rt: &crate::LspRuntime,
    path: &std::path::PathBuf,
    lang: &tree_sitter::Language,
) {
    // Create tree-sitter.
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(lang).unwrap();

    // Parse as AST-Tree.
    let content = std::fs::read_to_string(path).unwrap();
    let tree = parser.parse(&content, None).unwrap();

    let mut recurse = true;
    let mut finished = false;
    let mut cursor = tree.walk();

    while !finished {
        if recurse && cursor.goto_first_child() {
            recurse = true;

            pick_node(rt, &content, &mut cursor);
        } else {
            if cursor.goto_next_sibling() {
                recurse = true;

                pick_node(rt, &content, &mut cursor);
            } else if cursor.goto_parent() {
                recurse = false;
            } else {
                finished = true;
            }
        }
    }
}

fn pick_node(_rt: &crate::LspRuntime, source: &String, cursor: &mut tree_sitter::TreeCursor) {
    let node = cursor.node();
    tracing::trace!(
        "{}",
        format!(
            "{}`{}`({}): {}",
            "  ".repeat(cursor.depth() as usize),
            node.kind(),
            node.kind_id(),
            node.utf8_text(source.as_bytes()).unwrap()
        )
    );

    match node.kind_id().try_into() {
        Ok(crate::method::TreeSitterNodeKind::PreprocFunctionDef) => {}
        Err(_) => {}
    }
}

/// Safe workspace folders from client initialize params.
///
/// # Arguments
///
/// + `dst` - A mut reference to Runtime.
/// + `src` - Reference to InitializeParams
fn copy_workspace_folder(dst: &mut crate::LspRuntime, src: &InitializeParams) {
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
