pub struct SyntaxTreeC {}

impl SyntaxTreeC {
    pub fn new() -> Self {
        Self {}
    }
}

impl crate::syntax::SyntaxTree for SyntaxTreeC {
    fn parser(&self, source: &String, db: &crate::db::SqliteClient) -> crate::Result<()> {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&tree_sitter_c::language()).unwrap();

        let tree = parser.parse(source, None).unwrap();
        let mut cursor = tree.walk();

        parser_ast(source, &mut cursor, db).unwrap();

        Ok(())
    }
}

fn parser_ast(
    source: &String,
    cursor: &mut tree_sitter::TreeCursor,
    _db: &crate::db::SqliteClient,
) -> crate::Result<()> {
    let mut recurse = true;
    let mut finished = false;

    while !finished {
        if recurse && cursor.goto_first_child() {
            recurse = true;

            pick_node(&source, cursor);
        } else {
            if cursor.goto_next_sibling() {
                recurse = true;

                pick_node(&source, cursor);
            } else if cursor.goto_parent() {
                recurse = false;
            } else {
                finished = true;
            }
        }
    }

    Ok(())
}

fn pick_node(source: &String, cursor: &mut tree_sitter::TreeCursor) {
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
