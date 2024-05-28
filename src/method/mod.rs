pub mod initialize;

/// LSP error code that [`tower_lsp::jsonrpc::ErrorCode`] not defined.
#[allow(dead_code)]
pub enum LspErrorCode {
    /// Error code indicating that a server received a notification or
    /// request before the server has received the `initialize` request.
    ServerNotInitialized,

    /// A request failed but it was syntactically correct, e.g the
    /// method name was known and the parameters were valid. The error
    /// message should contain human readable information about why
    /// the request failed.
    RequestFailed,
}

impl LspErrorCode {
    pub const fn code(&self) -> i64 {
        match self {
            LspErrorCode::ServerNotInitialized => -32002,
            LspErrorCode::RequestFailed => -32803,
        }
    }
}
