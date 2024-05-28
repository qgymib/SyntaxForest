pub mod goto_definition;
pub mod initialize;
pub mod shutdown;

#[derive(Debug, Clone, Default)]
pub struct LspBackend {
    /// The name of the program.
    pub prog_name: String,

    /// The version of the program.
    pub prog_version: String,

    /// The list of workspace folders.
    pub workspace_folders: Vec<lsp_types::WorkspaceFolder>,

    /// The database.
    pub db: Option<std::rc::Rc<rusqlite::Connection>>,
}
