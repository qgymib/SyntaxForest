mod c;

pub trait SyntaxTree {
    fn parser(&self, source: &String, db: &crate::db::SqliteClient) -> crate::Result<()>;
}

#[derive(Debug, Default)]
struct SyntaxParserInner {
    language_table: std::collections::BTreeMap<String, fn() -> Box<dyn SyntaxTree>>,
    file_association_table: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct SyntaxParser {
    inner: std::sync::Arc<std::sync::Mutex<SyntaxParserInner>>,
}

impl SyntaxParser {
    pub fn new() -> SyntaxParser {
        let mut inner = SyntaxParserInner::default();

        // Register syntax parsers.
        inner.language_table.insert("C".to_string(), || {
            Box::new(crate::syntax::c::SyntaxTreeC::new())
        });

        // Register file association.
        inner
            .file_association_table
            .insert(".c".to_string(), "C".to_string());

        let parser = SyntaxParser {
            inner: std::sync::Arc::new(std::sync::Mutex::new(inner)),
        };
        return parser;
    }

    pub fn filter_file_suffix(
        &self,
        file_list: &Vec<crate::db::FileInfo>,
    ) -> Vec<crate::db::FileInfo> {
        let mut ret = Vec::new();
        for file in file_list {
            let path = &file.path;
            if self.is_match_extension(path) {
                ret.push(file.clone());
            }
        }
        return ret;
    }

    pub fn parser(
        &self,
        path: &std::path::PathBuf,
        db: &crate::db::SqliteClient,
    ) -> crate::Result<()> {
        let inner = self.inner.lock().unwrap();

        for (k, lang) in &inner.file_association_table {
            let file_path = path.to_str().unwrap();
            if file_path.ends_with(k.as_str()) {
                let content = std::fs::read_to_string(path).unwrap();

                let p = inner.language_table.get(lang).unwrap();
                p().parser(&content, db).unwrap();
            }
        }

        Ok(())
    }

    fn is_match_extension(&self, path: &std::path::PathBuf) -> bool {
        let path = path.to_str().unwrap();
        let inner = self.inner.lock().unwrap();

        for (k, _) in &inner.file_association_table {
            if path.ends_with(k.as_str()) {
                return true;
            }
        }
        return false;
    }
}
