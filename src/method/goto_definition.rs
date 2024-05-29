pub fn goto_definition(
    _rt: &mut crate::LspRuntime,
    _params: lsp_types::GotoDefinitionParams,
) -> Result<lsp_server::Response, Box<dyn std::error::Error + Sync + Send>> {
    return Ok(lsp_server::Response {
        id: 0.into(),
        result: None,
        error: Some(lsp_server::ResponseError {
            code: lsp_server::ErrorCode::MethodNotFound as i32,
            message: format!("method not found"),
            data: None,
        }),
    });
}
