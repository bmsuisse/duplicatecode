//! Unit extraction: functions, methods, classes and components, with a normalized token stream.

use crate::fingerprint::kgram_hashes;
use crate::lang::Lang;
use crate::naming::split_identifier;
use std::collections::{BTreeMap, BTreeSet};
use tree_sitter::{Node, Parser};

pub const KGRAM: usize = 4;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Unit {
    pub file: String,
    pub lang: Lang,
    pub name: String,
    pub kind: String,
    /// 1-based inclusive line range.
    pub start_line: u32,
    pub end_line: u32,
    /// Identifier/literal-normalized leaf tokens (comments, docstrings, type annotations dropped).
    #[serde(skip)]
    pub tokens: Vec<String>,
    /// Lower-cased names of called functions/methods.
    pub callees: BTreeSet<String>,
    #[serde(skip)]
    pub fingerprint: BTreeSet<u64>,
    /// Looser fingerprint: 2-grams of the same token stream.
    #[serde(skip)]
    pub fingerprint2: BTreeSet<u64>,
    /// Notable literals (numbers, short strings), excluding trivial ones like 0/1/"".
    #[serde(skip)]
    pub literals: BTreeSet<String>,
    /// Called function names plus accessed attribute/member names, lower-cased.
    #[serde(skip)]
    pub api: BTreeSet<String>,
    /// Histogram of syntax-node kinds (bag of structure).
    #[serde(skip)]
    pub kinds: BTreeMap<String, u32>,
    #[serde(skip)]
    pub name_parts: BTreeSet<String>,
}

impl Unit {
    pub fn token_count(&self) -> usize {
        self.tokens.len()
    }
}

pub fn extract_units(file: &str, lang: Lang, source: &str) -> Vec<Unit> {
    let mut parser = Parser::new();
    if parser.set_language(&lang.ts_language()).is_err() {
        return Vec::new();
    }
    let Some(tree) = parser.parse(source, None) else {
        return Vec::new();
    };
    let mut units = Vec::new();
    let mut ctx = Ctx { file, lang, src: source.as_bytes(), units: &mut units };
    ctx.walk(tree.root_node());
    units
}

#[derive(Default)]
struct Features {
    tokens: Vec<String>,
    callees: BTreeSet<String>,
    literals: BTreeSet<String>,
    api: BTreeSet<String>,
    kinds: BTreeMap<String, u32>,
}

struct Ctx<'a> {
    file: &'a str,
    lang: Lang,
    src: &'a [u8],
    units: &'a mut Vec<Unit>,
}

impl Ctx<'_> {
    fn text(&self, n: Node) -> String {
        n.utf8_text(self.src).unwrap_or("").to_string()
    }

    fn walk(&mut self, node: Node) {
        if let Some((kind, name, unit_node)) = self.classify(node) {
            self.emit(kind, name, unit_node);
            // Only classes are searched for member units; nested functions belong to their parent.
            if kind == "class" {
                for i in 0..node.child_count() {
                    if let Some(c) = node.child(i) {
                        self.walk_class_members(c);
                    }
                }
            }
            return;
        }
        for i in 0..node.child_count() {
            if let Some(c) = node.child(i) {
                self.walk(c);
            }
        }
    }

    fn walk_class_members(&mut self, node: Node) {
        if let Some((kind, name, unit_node)) = self.classify(node) {
            if kind != "class" {
                self.emit("method", name, unit_node);
            }
            return;
        }
        for i in 0..node.child_count() {
            if let Some(c) = node.child(i) {
                self.walk_class_members(c);
            }
        }
    }

    /// Returns (kind, name, node whose span/tokens make up the unit).
    fn classify<'t>(&self, node: Node<'t>) -> Option<(&'static str, String, Node<'t>)> {
        let field_name = |n: Node| n.child_by_field_name("name").map(|x| self.text(x));
        match (self.lang, node.kind()) {
            (Lang::Python, "function_definition") => {
                Some(("function", field_name(node)?, node))
            }
            (Lang::Python, "class_definition") => Some(("class", field_name(node)?, node)),
            (_, "function_declaration" | "generator_function_declaration") => {
                Some(("function", field_name(node)?, node))
            }
            (_, "class_declaration" | "abstract_class_declaration") => {
                Some(("class", field_name(node)?, node))
            }
            (_, "method_definition") => Some(("function", field_name(node)?, node)),
            // const foo = () => ... / const foo = function () {...}; also class fields
            (Lang::TypeScript | Lang::Tsx, "variable_declarator" | "public_field_definition") => {
                let value = node.child_by_field_name("value")?;
                if matches!(value.kind(), "arrow_function" | "function_expression" | "function") {
                    Some(("function", field_name(node)?, node))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn emit(&mut self, kind: &str, name: String, node: Node) {
        let mut f = Features::default();
        self.collect(node, &mut f);
        let Features { tokens, callees, literals, api, kinds } = f;
        let fingerprint = kgram_hashes(&tokens, KGRAM);
        let fingerprint2 = kgram_hashes(&tokens, 2);
        let name_parts = split_identifier(&name).into_iter().collect();
        self.units.push(Unit {
            file: self.file.to_string(),
            lang: self.lang,
            name,
            kind: kind.to_string(),
            start_line: node.start_position().row as u32 + 1,
            end_line: node.end_position().row as u32 + 1,
            tokens,
            callees,
            fingerprint,
            fingerprint2,
            literals,
            api,
            kinds,
            name_parts,
        });
    }

    fn collect(&self, node: Node, f: &mut Features) {
        let kind = node.kind();
        if matches!(
            kind,
            "comment"
                // TypeScript type-level syntax
                | "type_annotation"
                | "type_parameters"
                | "type_arguments"
                | "type_predicate_annotation"
                | "asserts_annotation"
                // Python annotations
                | "type"
                | "type_parameter"
        ) {
            return;
        }
        if self.lang == Lang::Python && kind == "expression_statement" && is_docstring(node) {
            return;
        }
        match kind {
            "call" | "call_expression" => {
                if let Some(func) = node.child_by_field_name("function") {
                    if let Some(n) = callee_name(func, self.src) {
                        let n = n.to_lowercase();
                        f.api.insert(n.clone());
                        f.callees.insert(n);
                    }
                }
            }
            "attribute" | "member_expression" => {
                let field = if kind == "attribute" { "attribute" } else { "property" };
                if let Some(a) = node.child_by_field_name(field) {
                    f.api.insert(self.text(a).to_lowercase());
                }
            }
            _ => {}
        }
        match kind {
            "string" | "template_string" | "string_literal" | "concatenated_string" => {
                f.tokens.push("STR".into());
                let t = self.text(node);
                let t = t.trim_matches(|c| c == '"' || c == '\'' || c == '`');
                if !t.is_empty() && t.len() <= 24 && !t.contains('{') {
                    f.literals.insert(t.to_lowercase());
                }
                return;
            }
            "integer" | "float" | "number" => {
                f.tokens.push("NUM".into());
                let t = self.text(node);
                if !matches!(t.as_str(), "0" | "1" | "2" | "-1") {
                    f.literals.insert(t);
                }
                return;
            }
            "identifier" | "property_identifier" | "shorthand_property_identifier"
            | "shorthand_property_identifier_pattern" | "type_identifier"
            | "private_property_identifier" => {
                f.tokens.push("ID".into());
                return;
            }
            _ => {}
        }
        if node.child_count() == 0 {
            f.tokens.push(kind.to_string());
            return;
        }
        if node.is_named() {
            *f.kinds.entry(kind.to_string()).or_insert(0) += 1;
        }
        for i in 0..node.child_count() {
            if let Some(c) = node.child(i) {
                self.collect(c, f);
            }
        }
    }
}

fn is_docstring(node: Node) -> bool {
    node.named_child_count() == 1 && node.named_child(0).is_some_and(|c| c.kind() == "string")
}

fn callee_name(f: Node, src: &[u8]) -> Option<String> {
    let text = |n: Node| n.utf8_text(src).ok().map(str::to_string);
    match f.kind() {
        "identifier" | "property_identifier" => text(f),
        "attribute" => f.child_by_field_name("attribute").and_then(text),
        "member_expression" => f.child_by_field_name("property").and_then(text),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn python_functions_methods_and_classes() {
        let src = "def f(a: int) -> int:\n    \"\"\"doc\"\"\"\n    return g(a) + 1\n\nclass C:\n    def m(self):\n        return 2\n";
        let u = extract_units("x.py", Lang::Python, src);
        let names: Vec<_> = u.iter().map(|u| (u.kind.as_str(), u.name.as_str())).collect();
        assert_eq!(names, [("function", "f"), ("class", "C"), ("method", "m")]);
        assert!(u[0].callees.contains("g"));
        // docstring and annotations are dropped
        assert!(!u[0].tokens.iter().any(|t| t == "STR"));
    }

    #[test]
    fn typescript_arrow_and_class() {
        let src = "export const f = (a: number): number => g(a) + 1;\nexport class K { m(): void {} }\n";
        let u = extract_units("x.ts", Lang::TypeScript, src);
        let names: Vec<_> = u.iter().map(|u| (u.kind.as_str(), u.name.as_str())).collect();
        assert_eq!(names, [("function", "f"), ("class", "K"), ("method", "m")]);
    }

    #[test]
    fn renaming_does_not_change_tokens() {
        let a = extract_units("a.py", Lang::Python, "def f(x):\n    return x + 1\n");
        let b = extract_units("b.py", Lang::Python, "def other(value):\n    return value + 2\n");
        assert_eq!(a[0].tokens, b[0].tokens);
    }
}
