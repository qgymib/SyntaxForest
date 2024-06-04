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
    let client = crate::db::SqliteClient::new(conn)?;

    let mut rt = LspRuntime {
        workspace_folders: vec![],
        db: client,
        parser: crate::syntax::SyntaxParser::new(),
    };

    // Parse the initialization parameters.
    let initialization_params: lsp_types::InitializeParams =
        serde_json::from_value(initialization_params)?;
    copy_workspace_folder(&mut rt, &initialization_params);

    let mut file_list = Vec::new();
    for folder in &rt.workspace_folders {
        let file_path = folder.uri.to_file_path().unwrap();
        let mut cwd_file_list = crate::utils::path::walk_with_gitignore(file_path)?;
        file_list.append(&mut cwd_file_list);
    }
    let file_list = rt.parser.filter_file_suffix(&file_list);

    rt.db.startup_scan(&file_list)?;

    trigger_tree_sitter(&mut rt);

    Ok(rt)
}

fn trigger_tree_sitter(rt: &mut crate::LspRuntime) {
    let files = rt.db.pending_analysis().unwrap();

    for file in files {
        rt.parser.parser(&file.path, &rt.db).unwrap();
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
