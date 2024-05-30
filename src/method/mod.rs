pub mod goto_definition;
pub mod initialize;
pub mod shutdown;

/// TreeSitter node kind.
#[derive(Debug, Copy, Clone)]
pub enum TreeSitterNodeKind {
    PreprocFunctionDef = 158,
}

impl std::convert::TryFrom<u16> for TreeSitterNodeKind {
    type Error = u16;

    /// Convert from `u16` to `TreeSitterNodeKind`.
    ///
    /// # Arguments
    ///
    /// + `value` - `u16` to convert.
    ///
    /// # Returns
    ///
    /// + `TreeSitterNodeKind` converted from `u16`.
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            158 => Ok(TreeSitterNodeKind::PreprocFunctionDef),
            unmatched => Err(unmatched),
        }
    }
}
