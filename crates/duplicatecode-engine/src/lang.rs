use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    Python,
    TypeScript,
    Tsx,
}

impl Lang {
    pub fn from_path(path: &Path) -> Option<Lang> {
        match path.extension()?.to_str()? {
            "py" => Some(Lang::Python),
            "ts" | "mts" | "cts" => Some(Lang::TypeScript),
            "tsx" => Some(Lang::Tsx),
            _ => None,
        }
    }

    pub fn ts_language(self) -> tree_sitter::Language {
        match self {
            Lang::Python => tree_sitter_python::LANGUAGE.into(),
            Lang::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Lang::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
        }
    }

    /// Python vs. the TypeScript family; units are only compared within a family.
    pub fn family(self) -> &'static str {
        match self {
            Lang::Python => "python",
            Lang::TypeScript | Lang::Tsx => "typescript",
        }
    }
}
