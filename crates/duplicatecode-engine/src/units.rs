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
    /// Multiset of normalized statement hashes (order-insensitive, header-only for compound statements).
    #[serde(skip)]
    pub stmts_exact: BTreeMap<u64, u32>,
    /// Multiset of coarse statement shapes (e.g. `expression_statement>assignment>call`).
    #[serde(skip)]
    pub stmts_shape: BTreeMap<u64, u32>,
    /// Statement shapes in source order (for order-aware alignment).
    #[serde(skip)]
    pub shape_seq: Vec<u64>,
    /// Constructors, dunder methods, field-only classes and similar: high similarity there is
    /// expected noise.
    pub boilerplate: bool,
    /// Lives in test code (path or name convention).
    pub is_test: bool,
    /// Number of source lines.
    pub lines: u32,
    /// Unit-length embedding of the name (only when embeddings are enabled).
    #[serde(skip)]
    pub name_vec: Option<std::sync::Arc<[f32]>>,
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
    let mut ctx = Ctx {
        local: sql_locals(lang, tree.root_node(), source.as_bytes()),
        keep_output: false,
        file,
        lang,
        src: source.as_bytes(),
        units: &mut units,
    };
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

/// The whole file as a single unit (all top-level and nested code together).
pub fn extract_file_unit(file: &str, lang: Lang, source: &str) -> Option<Unit> {
    let mut parser = Parser::new();
    parser.set_language(&lang.ts_language()).ok()?;
    let pruned = prune_dead_toplevel(&mut parser, lang, source);
    let source = pruned.as_str();
    let tree = parser.parse(source, None)?;
    let mut units = Vec::new();
    let mut ctx = Ctx {
        local: sql_locals(lang, tree.root_node(), source.as_bytes()),
        keep_output: true,
        file,
        lang,
        src: source.as_bytes(),
        units: &mut units,
    };
    let stem = file.rsplit('/').next().unwrap_or(file);
    let stem = stem.split('.').next().unwrap_or(stem).to_string();
    ctx.emit("file", stem, tree.root_node());
    units.pop()
}

/// Expression without side effects: literals, names, operators over them, containers of them and
/// calls to a few pure builtins (`float('inf')`).
fn is_pure_expr(node: Node, src: &[u8]) -> bool {
    const PURE_CALLS: &[&str] = &[
        "int", "float", "str", "bool", "list", "set", "dict", "tuple", "range",
    ];
    match node.kind() {
        "integer"
        | "float"
        | "string"
        | "true"
        | "false"
        | "none"
        | "null"
        | "number"
        | "template_string"
        | "identifier"
        | "unary_operator"
        | "binary_operator"
        | "unary_expression"
        | "binary_expression"
        | "parenthesized_expression"
        | "tuple"
        | "list"
        | "dictionary"
        | "set"
        | "pair"
        | "array"
        | "object"
        | "string_content"
        | "string_start"
        | "string_end"
        | "escape_sequence" => node
            .named_children(&mut node.walk())
            .all(|c| is_pure_expr(c, src)),
        "call" => {
            let f = node.child_by_field_name("function");
            let name = f.and_then(|f| f.utf8_text(src).ok()).unwrap_or("");
            PURE_CALLS.contains(&name)
                && node.child_by_field_name("arguments").is_none_or(|a| {
                    a.named_children(&mut a.walk())
                        .all(|c| is_pure_expr(c, src))
                })
        }
        "interpolation" => false,
        _ => false,
    }
}

/// Names a top-level statement introduces (function/class names, import bindings); empty when the
/// statement must be kept (executable code, wildcard imports, ...).
fn defined_names(node: Node, lang: Lang, src: &[u8]) -> Vec<String> {
    let text = |n: Node| n.utf8_text(src).unwrap_or("").to_string();
    let field = |n: Node, f: &str| n.child_by_field_name(f).map(text);
    match (lang, node.kind()) {
        (Lang::Python, "function_definition" | "class_definition") => {
            field(node, "name").into_iter().collect()
        }
        (Lang::Python, "decorated_definition") => node
            .child_by_field_name("definition")
            .map(|d| defined_names(d, lang, src))
            .unwrap_or_default(),
        // `MOD = 10**9 + 7` / `INF = float('inf')`: pure module-level constants nobody reads
        (Lang::Python, "expression_statement") => {
            let Some(a) = node.named_child(0).filter(|c| c.kind() == "assignment") else {
                return Vec::new();
            };
            match (
                a.child_by_field_name("left"),
                a.child_by_field_name("right"),
            ) {
                (Some(l), Some(r)) if l.kind() == "identifier" && is_pure_expr(r, src) => {
                    vec![text(l)]
                }
                _ => Vec::new(),
            }
        }
        (Lang::Python, "import_statement" | "import_from_statement") => {
            let mut cur = node.walk();
            let mut out = Vec::new();
            for n in node.children_by_field_name("name", &mut cur) {
                if n.kind() == "aliased_import" {
                    out.extend(field(n, "alias"));
                } else {
                    out.push(text(n).split('.').next().unwrap_or("").to_string());
                }
            }
            if node
                .children(&mut node.walk())
                .any(|c| c.kind() == "wildcard_import")
            {
                out.clear();
            }
            out
        }
        (Lang::TypeScript | Lang::Tsx, "function_declaration" | "class_declaration") => {
            field(node, "name").into_iter().collect()
        }
        (Lang::TypeScript | Lang::Tsx, "lexical_declaration") => {
            let mut cur = node.walk();
            let mut out = Vec::new();
            for d in node.named_children(&mut cur) {
                let is_fn = d.child_by_field_name("value").is_some_and(|v| {
                    matches!(v.kind(), "arrow_function" | "function_expression")
                        || is_pure_expr(v, src)
                });
                match (is_fn, field(d, "name")) {
                    (true, Some(n)) => out.push(n),
                    _ => return Vec::new(), // top-level state/side effects stay
                }
            }
            out
        }
        (Lang::TypeScript | Lang::Tsx, "import_statement") => {
            let mut out = Vec::new();
            let mut stack = vec![node];
            while let Some(n) = stack.pop() {
                if n.kind() == "identifier" {
                    out.push(text(n));
                } else if n.kind() != "string" {
                    stack.extend(n.named_children(&mut n.walk()));
                }
            }
            out
        }
        _ => Vec::new(),
    }
}

/// Blank out top-level functions, classes and imports that nothing else in the file references
/// (competitive-programming templates, unused helpers), repeating until nothing more is dead, so
/// that whole-file comparison looks at the code that actually runs. Line structure is kept.
fn prune_dead_toplevel(parser: &mut Parser, lang: Lang, source: &str) -> String {
    let mut cur = source.to_string();
    for _ in 0..4 {
        let Some(tree) = parser.parse(&cur, None) else {
            break;
        };
        let src = cur.as_bytes();
        let root = tree.root_node();
        // every identifier occurrence: (text, byte offset)
        let mut idents: Vec<(&str, usize)> = Vec::new();
        let mut stack = vec![root];
        while let Some(n) = stack.pop() {
            if n.child_count() == 0 && n.kind().ends_with("identifier") {
                idents.push((n.utf8_text(src).unwrap_or(""), n.start_byte()));
            }
            stack.extend(n.children(&mut n.walk()));
        }
        let mut dead: Vec<(usize, usize)> = Vec::new();
        for c in root.named_children(&mut root.walk()) {
            let names = defined_names(c, lang, src);
            if names.is_empty() {
                continue;
            }
            let used = names.iter().any(|name| {
                idents
                    .iter()
                    .any(|(t, at)| t == name && !(c.start_byte()..c.end_byte()).contains(at))
            });
            if !used {
                dead.push((c.start_byte(), c.end_byte()));
            }
        }
        if dead.is_empty() {
            break;
        }
        let mut bytes = cur.into_bytes();
        for (a, b) in dead {
            for x in &mut bytes[a..b] {
                if *x != b'\n' {
                    *x = b' ';
                }
            }
        }
        cur = String::from_utf8(bytes).unwrap_or_default();
    }
    cur
}

fn strip_quotes(t: &str) -> &str {
    t.trim_matches(|c| matches!(c, '"' | '\'' | '`' | '[' | ']'))
}

/// Alias and CTE names of a SQL file (lower-cased).
fn sql_locals(lang: Lang, root: Node, src: &[u8]) -> std::collections::HashSet<String> {
    let mut out = std::collections::HashSet::new();
    if lang != Lang::Sql {
        return out;
    }
    let txt = |n: Node| strip_quotes(n.utf8_text(src).unwrap_or("")).to_lowercase();
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        if let Some(a) = n.child_by_field_name("alias") {
            out.insert(txt(a));
        }
        if n.kind() == "cte" {
            if let Some(id) = n.named_child(0).filter(|c| c.kind() == "identifier") {
                out.insert(txt(id));
            }
        }
        stack.extend(n.children(&mut n.walk()));
    }
    out
}

struct Ctx<'a> {
    /// SQL: lower-cased alias and CTE names; they are abstracted like local variables.
    local: std::collections::HashSet<String>,
    /// Whole-file (script) mode: print/console output is the program's result, not debug noise.
    keep_output: bool,
    file: &'a str,
    lang: Lang,
    src: &'a [u8],
    units: &'a mut Vec<Unit>,
}

impl Ctx<'_> {
    fn first_descendant<'t>(&self, node: Node<'t>, kind: &str) -> Option<Node<'t>> {
        let mut stack = vec![node];
        while let Some(n) = stack.pop() {
            if n.kind() == kind {
                return Some(n);
            }
            let mut kids: Vec<_> = n.children(&mut n.walk()).collect();
            kids.reverse();
            stack.extend(kids);
        }
        None
    }

    fn text(&self, n: Node) -> String {
        n.utf8_text(self.src).unwrap_or("").to_string()
    }

    fn walk(&mut self, node: Node) {
        // bodyless signatures (interfaces) and enums are not code units
        if self.lang == Lang::CSharp
            && matches!(node.kind(), "interface_declaration" | "enum_declaration")
        {
            return;
        }
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
            (Lang::Sql, "statement") => {
                let name = self
                    .first_descendant(node, "object_reference")
                    .filter(|_| {
                        node.named_child(0).is_some_and(|c| {
                            c.kind().starts_with("create") || c.kind().starts_with("insert")
                        })
                    })
                    .map(|o| strip_quotes(&self.text(o)).to_lowercase())
                    .unwrap_or_else(|| "query".into());
                Some(("statement", name, node))
            }
            (Lang::Python, "function_definition") => Some(("function", field_name(node)?, node)),
            (Lang::Python, "class_definition") => Some(("class", field_name(node)?, node)),
            (_, "function_declaration" | "generator_function_declaration") => {
                Some(("function", field_name(node)?, node))
            }
            (_, "class_declaration" | "abstract_class_declaration") => {
                Some(("class", field_name(node)?, node))
            }
            (_, "method_definition") => Some(("function", field_name(node)?, node)),
            (
                Lang::CSharp,
                "struct_declaration" | "record_declaration" | "record_struct_declaration",
            ) => Some(("class", field_name(node)?, node)),
            (Lang::CSharp, "method_declaration" | "local_function_statement") => {
                Some(("function", field_name(node)?, node))
            }
            (Lang::CSharp, "constructor_declaration") => {
                Some(("function", "constructor".into(), node))
            }
            // const foo = () => ... / const foo = function () {...}; also class fields
            (Lang::TypeScript | Lang::Tsx, "variable_declarator" | "public_field_definition") => {
                let value = node.child_by_field_name("value")?;
                if matches!(
                    value.kind(),
                    "arrow_function" | "function_expression" | "function"
                ) {
                    Some(("function", field_name(node)?, node))
                } else if node.kind() == "variable_declarator"
                    && matches!(
                        value.kind(),
                        "new_expression" | "call_expression" | "object"
                    )
                    && is_module_level(node)
                {
                    // e.g. `export const queryClient = new QueryClient({...})`
                    Some(("value", field_name(node)?, node))
                } else {
                    None
                }
            }
            (Lang::Python, "expression_statement")
                if node.parent().is_some_and(|p| p.kind() == "module") =>
            {
                let asg = node.named_child(0).filter(|a| a.kind() == "assignment")?;
                let left = asg
                    .child_by_field_name("left")
                    .filter(|l| l.kind() == "identifier")?;
                let right = asg.child_by_field_name("right")?;
                matches!(right.kind(), "call" | "dictionary" | "list" | "set")
                    .then(|| ("value", self.text(left), node))
            }
            _ => None,
        }
    }

    fn emit(&mut self, kind: &str, name: String, node: Node) {
        // one-line module constants (`_DSN = ...`) repeat everywhere and mean nothing
        if kind == "value" && node.end_position().row - node.start_position().row + 1 < 4 {
            return;
        }
        let mut f = Features::default();
        self.collect_features(node, &mut f);
        self.norm_tokens(node, &mut f.tokens, false);
        let Features {
            tokens,
            callees,
            literals,
            api,
            kinds,
        } = f;
        let fingerprint = kgram_hashes(&tokens, KGRAM);
        let fingerprint2 = kgram_hashes(&tokens, 2);
        let mut stmts = Vec::new();
        self.statements(node, &mut stmts);
        let mut stmts_exact = BTreeMap::new();
        let mut stmts_shape = BTreeMap::new();
        let mut shape_seq = Vec::new();
        for st in &stmts {
            *stmts_exact.entry(hash_of(&st.tokens)).or_insert(0) += 1;
            let sh = hash_of(&st.shape);
            *stmts_shape.entry(sh).or_insert(0) += 1;
            shape_seq.push(sh);
        }
        let boilerplate = is_boilerplate(&name) || (kind == "class" && self.fields_only(node));
        let is_test =
            is_test_path(self.file) || name.starts_with("test_") || name.starts_with("Test");
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
            stmts_exact,
            stmts_shape,
            shape_seq,
            boilerplate,
            is_test,
            lines: (node.end_position().row - node.start_position().row + 1) as u32,
            name_vec: None,
            name_parts,
        });
    }

    /// A class that only declares fields (data model / DTO): no methods, no logic.
    fn fields_only(&self, class: Node) -> bool {
        let Some(body) = class.child_by_field_name("body") else {
            return false;
        };
        let mut fields = 0;
        for i in 0..body.named_child_count() {
            let Some(m) = body.named_child(i as u32) else {
                continue;
            };
            match m.kind() {
                "comment" | "pass_statement" | "decorator" => {}
                "expression_statement" => match m.named_child(0).map(|e| e.kind()) {
                    Some("assignment" | "string" | "ellipsis" | "concatenated_string") => {
                        fields += 1
                    }
                    _ => return false,
                },
                "field_declaration" | "property_declaration" if self.lang == Lang::CSharp => {
                    if m.children(&mut m.walk())
                        .any(|c| c.kind() == "arrow_expression_clause")
                        || m.children(&mut m.walk()).any(|c| {
                            c.kind() == "accessor_list"
                                && c.children(&mut c.walk()).any(|a| {
                                    a.kind() == "accessor_declaration"
                                        && a.child_by_field_name("body").is_some()
                                })
                        })
                    {
                        return false;
                    }
                    fields += 1;
                }
                "public_field_definition" | "property_signature" => {
                    if m.child_by_field_name("value").is_some_and(|v| {
                        matches!(
                            v.kind(),
                            "arrow_function" | "function_expression" | "function"
                        )
                    }) {
                        return false;
                    }
                    fields += 1;
                }
                _ => return false,
            }
        }
        fields > 0
    }

    // ---- normalization -------------------------------------------------------------------

    /// Nodes that never take part in comparison: comments, type-level syntax, docstrings and
    /// debug output (print/logging/console calls).
    fn is_ignored(&self, node: Node) -> bool {
        let kind = node.kind();
        TYPE_KINDS.contains(&kind)
            || (self.lang == Lang::Python && kind == "expression_statement" && is_docstring(node))
            || (!self.keep_output && self.is_debug(node))
    }

    fn is_debug(&self, node: Node) -> bool {
        match node.kind() {
            "debugger_statement" => true,
            "expression_statement" if self.lang == Lang::CSharp => {
                // Console.WriteLine(..), Debug.WriteLine(..), _logger.LogInformation(..)
                let Some(f) = node
                    .named_child(0)
                    .filter(|e| e.kind() == "invocation_expression")
                    .and_then(|e| e.child_by_field_name("function"))
                    .filter(|f| f.kind() == "member_access_expression")
                else {
                    return false;
                };
                let obj = f
                    .child_by_field_name("expression")
                    .map(|o| match o.kind() {
                        "member_access_expression" => o
                            .child_by_field_name("name")
                            .map(|x| self.text(x))
                            .unwrap_or_default(),
                        _ => self.text(o),
                    })
                    .unwrap_or_default();
                let method = f
                    .child_by_field_name("name")
                    .map(|x| self.text(x))
                    .unwrap_or_default();
                matches!(
                    obj.to_lowercase().as_str(),
                    "console" | "debug" | "trace" | "logger" | "_logger" | "log" | "_log"
                ) && (method.starts_with("Write") || method.starts_with("Log"))
            }
            "expression_statement" => {
                let Some(e) = node.named_child(0) else {
                    return false;
                };
                if !matches!(e.kind(), "call" | "call_expression") {
                    return false;
                }
                let Some(f) = e.child_by_field_name("function") else {
                    return false;
                };
                match f.kind() {
                    "identifier" => matches!(self.text(f).as_str(), "print" | "pprint"),
                    "attribute" | "member_expression" => {
                        let Some(obj) = f.child_by_field_name("object") else {
                            return false;
                        };
                        let name = match obj.kind() {
                            "identifier" => self.text(obj),
                            "attribute" | "member_expression" => obj
                                .child_by_field_name(if obj.kind() == "attribute" {
                                    "attribute"
                                } else {
                                    "property"
                                })
                                .map(|x| self.text(x))
                                .unwrap_or_default(),
                            _ => String::new(),
                        };
                        matches!(
                            name.to_lowercase().as_str(),
                            "logging" | "logger" | "log" | "console" | "_logger" | "_log"
                        )
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// `x = expr` immediately followed by `return x`  ==>  the value expression of `return expr`.
    fn temp_return<'t>(&self, a: Node<'t>, b: Node<'t>) -> Option<Node<'t>> {
        if self.lang == Lang::CSharp || b.kind() != "return_statement" || b.named_child_count() != 1
        {
            return None;
        }
        let (name, value) = if self.lang == Lang::Python {
            if a.kind() != "expression_statement" {
                return None;
            }
            let asg = a.named_child(0).filter(|x| x.kind() == "assignment")?;
            (
                asg.child_by_field_name("left")?,
                asg.child_by_field_name("right")?,
            )
        } else {
            if a.kind() != "lexical_declaration" || a.named_child_count() != 1 {
                return None;
            }
            let d = a
                .named_child(0)
                .filter(|x| x.kind() == "variable_declarator")?;
            (
                d.child_by_field_name("name")?,
                d.child_by_field_name("value")?,
            )
        };
        if matches!(
            value.kind(),
            "list_comprehension" | "set_comprehension" | "dictionary_comprehension"
        ) {
            return None; // keep the loop-shaped form (see comprehension_tokens)
        }
        let ret = b.named_child(0)?;
        (name.kind() == "identifier"
            && ret.kind() == "identifier"
            && self.text(name) == self.text(ret))
        .then_some(value)
    }

    /// Normalized leaf tokens of `node`. Identifiers/literals are abstracted; comprehensions are
    /// rendered as the equivalent loop; augmented assignment as plain assignment; a temp variable
    /// returned right after its definition is inlined. With `skip_blocks`, nested blocks are left
    /// out (statement headers).
    fn norm_tokens(&self, node: Node, out: &mut Vec<String>, skip_blocks: bool) {
        let kind = node.kind();
        if self.is_ignored(node) || (skip_blocks && matches!(kind, "block" | "statement_block")) {
            return;
        }
        match kind {
            _ if self.lang == Lang::Sql && node.child_count() == 0 => self.sql_leaf(node, out),
            "string"
            | "template_string"
            | "string_literal"
            | "concatenated_string"
            | "verbatim_string_literal"
            | "raw_string_literal"
            | "interpolated_string_expression" => out.push("STR".into()),
            "integer" | "float" | "number" | "integer_literal" | "real_literal" => {
                out.push("NUM".into())
            }
            "identifier"
            | "property_identifier"
            | "shorthand_property_identifier"
            | "shorthand_property_identifier_pattern"
            | "type_identifier"
            | "private_property_identifier" => out.push("ID".into()),
            "generic_name" if self.lang == Lang::CSharp => out.push("ID".into()),
            "list_comprehension"
            | "set_comprehension"
            | "dictionary_comprehension"
            | "generator_expression" => self.comprehension_tokens(node, out),
            // `return [e for ...]` reads like `r = []; for ...: r.append(e); return r`
            "return_statement"
                if self.lang == Lang::Python
                    && node.named_child(0).is_some_and(|e| {
                        matches!(
                            e.kind(),
                            "list_comprehension" | "set_comprehension" | "dictionary_comprehension"
                        )
                    }) =>
            {
                out.extend(["ID", "="].map(String::from));
                if let Some(e) = node.named_child(0) {
                    self.comprehension_tokens(e, out);
                }
                out.extend(["return", "ID"].map(String::from));
            }
            "augmented_assignment" | "augmented_assignment_expression" => {
                match (
                    node.child_by_field_name("left"),
                    node.child_by_field_name("operator"),
                    node.child_by_field_name("right"),
                ) {
                    (Some(l), Some(op), Some(r)) => {
                        self.norm_tokens(l, out, skip_blocks);
                        out.push("=".into());
                        self.norm_tokens(l, out, skip_blocks);
                        out.push(norm_leaf(op.kind().trim_end_matches('=')).to_string());
                        self.norm_tokens(r, out, skip_blocks);
                    }
                    _ => self.norm_children(node, out, skip_blocks),
                }
            }
            "block" | "statement_block" | "module" | "program" | "compilation_unit" => {
                let n = node.child_count();
                let mut i = 0;
                while i < n {
                    let Some(c) = node.child(i) else { break };
                    if let Some(v) = node.child(i + 1).and_then(|next| self.temp_return(c, next)) {
                        out.push("return".into());
                        self.norm_tokens(v, out, skip_blocks);
                        if self.lang != Lang::Python {
                            out.push(";".into());
                        }
                        i += 2;
                        continue;
                    }
                    self.norm_tokens(c, out, skip_blocks);
                    i += 1;
                }
            }
            _ if node.child_count() == 0 => out.push(norm_leaf(kind).to_string()),
            _ => self.norm_children(node, out, skip_blocks),
        }
    }

    /// SQL leaves: keywords lower-cased, aliases/CTE names/qualifiers abstracted, and table,
    /// column and function names kept (in SQL they are the semantics).
    fn sql_leaf(&self, node: Node, out: &mut Vec<String>) {
        let kind = node.kind();
        if let Some(k) = kind.strip_prefix("keyword_") {
            out.push(k.to_string());
            return;
        }
        match kind {
            "identifier" => {
                let low = strip_quotes(&self.text(node)).to_lowercase();
                let qualifier = node.parent().is_some_and(|p| {
                    p.kind() == "object_reference"
                        && p.parent().is_some_and(|g| g.kind() == "field")
                });
                if qualifier || self.local.contains(&low) {
                    out.push("ID".into());
                } else {
                    out.push(low);
                }
            }
            "literal" => {
                let t = self.text(node);
                let first = t.chars().next().unwrap_or(' ');
                if first.is_ascii_digit() || first == '-' || first == '.' {
                    out.push("NUM".into());
                } else if first == '\'' || first == '"' {
                    out.push("STR".into());
                } else {
                    out.push(t.to_lowercase());
                }
            }
            ";" => {} // statement terminators carry no meaning
            _ => out.push(norm_leaf(kind).to_string()),
        }
    }

    /// SQL features (callees, referenced tables/columns, literals); true = do not descend.
    fn sql_features(&self, node: Node, f: &mut Features) -> bool {
        let name_of = |n: Node| strip_quotes(&self.text(n)).to_lowercase();
        match node.kind() {
            "invocation" => {
                if let Some(o) = node
                    .named_child(0)
                    .filter(|c| c.kind() == "object_reference")
                {
                    let n = name_of(o);
                    f.api.insert(n.clone());
                    f.callees.insert(n);
                }
            }
            "relation" => {
                if let Some(o) = node
                    .named_child(0)
                    .filter(|c| c.kind() == "object_reference")
                {
                    let n = name_of(o);
                    if !self.local.contains(&n) {
                        f.api.insert(n);
                    }
                }
            }
            "field" => {
                if let Some(c) = node.child_by_field_name("name") {
                    let n = name_of(c);
                    if !self.local.contains(&n) {
                        f.api.insert(n);
                    }
                }
            }
            "literal" => {
                let t = self.text(node);
                let t = strip_quotes(&t);
                if !t.is_empty() && t.len() <= 24 && !matches!(t, "0" | "1" | "2" | "-1") {
                    f.literals.insert(t.to_lowercase());
                }
                return true;
            }
            _ => {}
        }
        false
    }

    fn norm_children(&self, node: Node, out: &mut Vec<String>, skip_blocks: bool) {
        for i in 0..node.child_count() {
            if let Some(c) = node.child(i) {
                self.norm_tokens(c, out, skip_blocks);
            }
        }
    }

    /// `[e for x in xs if c]` is rendered like `r = []; for x in xs: if c: r.append(e)`.
    fn comprehension_tokens(&self, node: Node, out: &mut Vec<String>) {
        let open: &[&str] = match node.kind() {
            "set_comprehension" => &["set", "(", ")"],
            "dictionary_comprehension" => &["{", "}"],
            _ => &["[", "]"],
        };
        out.extend(open.iter().map(|t| t.to_string()));
        for i in 0..node.child_count() {
            let Some(c) = node.child(i) else { continue };
            match c.kind() {
                "for_in_clause" => self.for_header(c, out),
                "if_clause" => self.if_header(c, out),
                _ => {}
            }
        }
        if let Some(body) = node.child_by_field_name("body") {
            self.append_stmt(node.kind(), body, out);
        }
    }

    fn for_header(&self, clause: Node, out: &mut Vec<String>) {
        out.push("for".into());
        if let Some(l) = clause.child_by_field_name("left") {
            self.norm_tokens(l, out, false);
        }
        out.push("in".into());
        if let Some(r) = clause.child_by_field_name("right") {
            self.norm_tokens(r, out, false);
        }
        out.push(":".into());
    }

    fn if_header(&self, clause: Node, out: &mut Vec<String>) {
        out.push("if".into());
        if let Some(c) = clause.named_child(0) {
            self.norm_tokens(c, out, false);
        }
        out.push(":".into());
    }

    fn append_stmt(&self, comp_kind: &str, body: Node, out: &mut Vec<String>) {
        if comp_kind == "dictionary_comprehension" {
            out.extend(["ID", "["].map(String::from));
            if let Some(k) = body.child_by_field_name("key") {
                self.norm_tokens(k, out, false);
            }
            out.extend(["]", "="].map(String::from));
            if let Some(v) = body.child_by_field_name("value") {
                self.norm_tokens(v, out, false);
            }
        } else {
            out.extend(["ID", ".", "ID", "("].map(String::from));
            self.norm_tokens(body, out, false);
            out.push(")".into());
        }
    }

    // ---- statements ------------------------------------------------------------------------

    /// Statements of a unit in source order: every child of a block, at any depth. A statement
    /// holds the tokens of its own header (nested blocks are their own statements).
    fn statements(&self, node: Node, out: &mut Vec<Stmt>) {
        let blockish = matches!(
            node.kind(),
            "block" | "statement_block" | "module" | "program" | "compilation_unit"
        ) || (self.lang == Lang::Sql
            && matches!(node.kind(), "statement" | "select" | "from" | "cte"));
        let n = node.child_count();
        let mut i = 0;
        while i < n {
            let Some(c) = node.child(i) else { break };
            if self.is_ignored(c) {
                i += 1;
                continue;
            }
            if blockish && c.is_named() {
                if let Some(v) = node.child(i + 1).and_then(|next| self.temp_return(c, next)) {
                    let mut tokens = vec!["return".to_string()];
                    self.norm_tokens(v, &mut tokens, true);
                    if self.lang != Lang::Python {
                        tokens.push(";".into());
                    }
                    let mut shape = vec!["return_statement".to_string()];
                    shape.extend(shape_of(v).into_iter().take(3));
                    out.push(Stmt { tokens, shape });
                    i += 2;
                    continue;
                }
                if !self.comprehension_stmts(c, out) {
                    let mut tokens = Vec::new();
                    self.norm_tokens(c, &mut tokens, true);
                    out.push(Stmt {
                        tokens,
                        shape: shape_of(c),
                    });
                }
            } else if node.kind() == "arrow_function"
                && node
                    .child_by_field_name("body")
                    .is_some_and(|b| b.id() == c.id())
                && c.kind() != "statement_block"
            {
                // expression-bodied arrow function: the body expression acts as a statement
                let mut tokens = Vec::new();
                self.norm_tokens(c, &mut tokens, true);
                out.push(Stmt {
                    tokens,
                    shape: vec!["return".into(), shape_of(c).join(">")],
                });
            }
            self.statements(c, out);
            i += 1;
        }
    }

    /// Python `x = [e for ...]` / `return [e for ...]` as the statements of the equivalent loop.
    fn comprehension_stmts(&self, c: Node, out: &mut Vec<Stmt>) -> bool {
        if self.lang != Lang::Python {
            return false;
        }
        let is_comp = |k: &str| {
            matches!(
                k,
                "list_comprehension" | "set_comprehension" | "dictionary_comprehension"
            )
        };
        let (target, comp, is_return) = match c.kind() {
            "expression_statement" => {
                let Some(a) = c.named_child(0).filter(|a| a.kind() == "assignment") else {
                    return false;
                };
                let (Some(l), Some(r)) = (
                    a.child_by_field_name("left"),
                    a.child_by_field_name("right"),
                ) else {
                    return false;
                };
                (Some(l), r, false)
            }
            "return_statement" => {
                let Some(e) = c.named_child(0) else {
                    return false;
                };
                (None, e, true)
            }
            _ => return false,
        };
        if !is_comp(comp.kind()) {
            return false;
        }
        let mut tokens = Vec::new();
        match target {
            Some(l) => self.norm_tokens(l, &mut tokens, true),
            None => tokens.push("ID".into()),
        }
        tokens.extend(["=", "[", "]"].map(String::from));
        out.push(Stmt {
            tokens,
            shape: ["expression_statement", "assignment", "list"]
                .map(String::from)
                .to_vec(),
        });
        for i in 0..comp.child_count() {
            let Some(cl) = comp.child(i) else { continue };
            let mut tokens = Vec::new();
            let mut shape;
            match cl.kind() {
                "for_in_clause" => {
                    self.for_header(cl, &mut tokens);
                    shape = vec!["for_statement".to_string()];
                    shape.extend(
                        cl.child_by_field_name("right")
                            .map(shape_of)
                            .unwrap_or_default()
                            .into_iter()
                            .take(3),
                    );
                }
                "if_clause" => {
                    self.if_header(cl, &mut tokens);
                    shape = vec!["if_statement".to_string()];
                    shape.extend(
                        cl.named_child(0)
                            .map(shape_of)
                            .unwrap_or_default()
                            .into_iter()
                            .take(3),
                    );
                }
                _ => continue,
            }
            out.push(Stmt { tokens, shape });
        }
        if let Some(body) = comp.child_by_field_name("body") {
            let mut tokens = Vec::new();
            self.append_stmt(comp.kind(), body, &mut tokens);
            out.push(Stmt {
                tokens,
                shape: ["expression_statement", "call", "attribute"]
                    .map(String::from)
                    .to_vec(),
            });
        }
        if is_return {
            out.push(Stmt {
                tokens: vec!["return".into(), "ID".into()],
                shape: vec!["return_statement".into()],
            });
        }
        true
    }

    // ---- features (API names, literals, node-kind histogram) ---------------------------------

    fn collect_features(&self, node: Node, f: &mut Features) {
        let kind = node.kind();
        if self.is_ignored(node) {
            return;
        }
        if self.lang == Lang::Sql && self.sql_features(node, f) {
            return;
        }
        match kind {
            "call" | "call_expression" | "invocation_expression" => {
                if let Some(func) = node.child_by_field_name("function") {
                    if let Some(n) = callee_name(func, self.src) {
                        let n = n.to_lowercase();
                        f.api.insert(n.clone());
                        f.callees.insert(n);
                    }
                }
            }
            "attribute" | "member_expression" | "member_access_expression" => {
                let field = match kind {
                    "attribute" => "attribute",
                    "member_access_expression" => "name",
                    _ => "property",
                };
                if let Some(a) = node.child_by_field_name(field) {
                    f.api.insert(self.text(a).to_lowercase());
                }
            }
            "string"
            | "template_string"
            | "string_literal"
            | "concatenated_string"
            | "verbatim_string_literal"
            | "raw_string_literal" => {
                let t = self.text(node);
                let t = t.trim_matches(|c| c == '"' || c == '\'' || c == '`');
                if !t.is_empty() && t.len() <= 24 && !t.contains('{') {
                    f.literals.insert(t.to_lowercase());
                }
                return;
            }
            "integer" | "float" | "number" | "integer_literal" | "real_literal" => {
                let t = self.text(node);
                if !matches!(t.as_str(), "0" | "1" | "2" | "-1") {
                    f.literals.insert(t);
                }
                return;
            }
            _ => {}
        }
        if node.child_count() == 0 {
            return;
        }
        // comprehensions count as the loop/branch nodes of their expanded form
        match kind {
            "list_comprehension"
            | "set_comprehension"
            | "dictionary_comprehension"
            | "generator_expression" => {}
            "for_in_clause" => *f.kinds.entry("for_statement".into()).or_insert(0) += 1,
            "if_clause" => *f.kinds.entry("if_statement".into()).or_insert(0) += 1,
            _ if node.is_named() => *f.kinds.entry(kind.to_string()).or_insert(0) += 1,
            _ => {}
        }
        for i in 0..node.child_count() {
            if let Some(c) = node.child(i) {
                self.collect_features(c, f);
            }
        }
    }
}

const TYPE_KINDS: &[&str] = &[
    "comment",
    // TypeScript type-level syntax
    "type_annotation",
    "type_parameters",
    "type_arguments",
    "type_predicate_annotation",
    "asserts_annotation",
    // C# type-level syntax
    "type_argument_list",
    "type_parameter_list",
    "type_parameter_constraints_clause",
    "nullable_type",
    "modifier",
    "predefined_type",
    // Python annotations
    "type",
    "type_parameter",
];

/// Operator/keyword spellings that mean the same across (and within) the supported languages.
fn norm_leaf(kind: &str) -> &str {
    match kind {
        "===" => "==",
        "!==" => "!=",
        "is" => "==",
        "and" => "&&",
        "or" => "||",
        "not" => "!",
        "none" => "null",
        "var" | "const" => "let",
        other => other,
    }
}

struct Stmt {
    tokens: Vec<String>,
    /// Coarse kind path, e.g. [expression_statement, assignment, call].
    shape: Vec<String>,
}

fn hash_of<T: std::hash::Hash + ?Sized>(t: &T) -> u64 {
    use std::hash::Hasher;
    let mut h = std::collections::hash_map::DefaultHasher::new();
    t.hash(&mut h);
    h.finish()
}

/// Kind path of a statement, following the most informative child (skipping bare identifiers,
/// comments and blocks) up to three levels deep.
fn shape_of(node: Node) -> Vec<String> {
    let mut path = vec![node.kind().to_string()];
    let mut cur = node;
    for _ in 0..3 {
        let next = ["right", "value", "argument", "condition"]
            .iter()
            .find_map(|f| cur.child_by_field_name(f))
            .or_else(|| {
                (0..cur.named_child_count())
                    .filter_map(|i| cur.named_child(i as u32))
                    .find(|c| {
                        !matches!(
                            c.kind(),
                            "identifier" | "comment" | "block" | "statement_block"
                        )
                    })
            });
        match next {
            Some(n) if !matches!(n.kind(), "block" | "statement_block") => {
                path.push(n.kind().to_string());
                cur = n;
            }
            _ => break,
        }
    }
    path
}

/// `const x = ...` / `export const x = ...` directly at the top level of a TS/JS file.
fn is_module_level(declarator: Node) -> bool {
    let Some(decl) = declarator
        .parent()
        .filter(|d| d.kind() == "lexical_declaration")
    else {
        return false;
    };
    let Some(up) = decl.parent() else {
        return false;
    };
    up.kind() == "program"
        || (up.kind() == "export_statement" && up.parent().is_some_and(|p| p.kind() == "program"))
}

fn is_test_path(file: &str) -> bool {
    let f = file.replace('\\', "/");
    let base = f.rsplit('/').next().unwrap_or(&f);
    f.split('/').any(|c| {
        matches!(c, "tests" | "test" | "__tests__" | "e2e" | "spec")
            || c.ends_with(".Tests")
            || c.ends_with(".Test")
    }) || base.ends_with("Tests.cs")
        || base.ends_with("Test.cs")
        || base.ends_with("Spec.cs")
        || base.starts_with("test_")
        || base == "conftest.py"
        || base.ends_with("_test.py")
        || [".test.", ".spec."].iter().any(|m| base.contains(m))
}

fn is_boilerplate(name: &str) -> bool {
    (name.starts_with("__") && name.ends_with("__")) || name == "constructor"
}

fn is_docstring(node: Node) -> bool {
    node.named_child_count() == 1 && node.named_child(0).is_some_and(|c| c.kind() == "string")
}

fn callee_name(f: Node, src: &[u8]) -> Option<String> {
    let text = |n: Node| n.utf8_text(src).ok().map(str::to_string);
    match f.kind() {
        "identifier" | "property_identifier" => text(f),
        "attribute" => f.child_by_field_name("attribute").and_then(text),
        "member_access_expression" => f.child_by_field_name("name").and_then(text),
        "member_expression" => f.child_by_field_name("property").and_then(text),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn sql_ignores_case_aliases_and_formatting_but_not_tables() {
        let f = |src: &str| extract_file_unit("q.sql", Lang::Sql, src).unwrap().tokens;
        let a = f("SELECT o.id, SUM(i.amt) AS total FROM orders o JOIN items i ON o.id = i.oid WHERE o.status = 'x' GROUP BY o.id;");
        let b = f("select\n  a.id,\n  sum(b.amt) as t\nfrom orders a\njoin items b on a.id = b.oid\nwhere a.status = 'y'\ngroup by a.id");
        assert_eq!(a, b);
        let c = f("SELECT o.id, SUM(i.amt) AS total FROM invoices o JOIN items i ON o.id = i.oid WHERE o.status = 'x' GROUP BY o.id;");
        assert_ne!(a, c);
        let u = extract_units(
            "q.sql",
            Lang::Sql,
            "SELECT 1 FROM t;\nCREATE VIEW v AS SELECT a FROM t;",
        );
        assert_eq!(u.len(), 2);
        assert_eq!(u[1].name, "v");
    }

    #[test]
    fn file_unit_ignores_unused_helpers_and_imports() {
        let core = "def main():\n    s = input()\n    print(s[::-1])\n\nmain()\n";
        let noisy = format!(
            "import math\nfrom heapq import heappush\n\ndef gcd(a, b):\n    return a if not b else gcd(b, a % b)\n\n\
             def lcm(a, b):\n    return a * b // gcd(a, b)\n\n{core}"
        );
        let a = extract_file_unit("a.py", Lang::Python, core).unwrap();
        let b = extract_file_unit("b.py", Lang::Python, &noisy).unwrap();
        assert_eq!(a.tokens, b.tokens);
        // helpers that are used stay
        let used = "def f(x):\n    return x + 1\n\nprint(f(2))\n";
        let u = extract_file_unit("c.py", Lang::Python, used).unwrap();
        assert!(u.tokens.len() > 8);
        let js = "import fs from 'fs';\nconst unused = () => 1;\nconsole.log(2);\n";
        let j = extract_file_unit("a.js", Lang::TypeScript, js).unwrap();
        let j2 = extract_file_unit("b.js", Lang::TypeScript, "console.log(2);\n").unwrap();
        assert_eq!(j.tokens, j2.tokens);
    }

    use super::*;

    #[test]
    fn python_functions_methods_and_classes() {
        let src = "def f(a: int) -> int:\n    \"\"\"doc\"\"\"\n    return g(a) + 1\n\nclass C:\n    def m(self):\n        return 2\n";
        let u = extract_units("x.py", Lang::Python, src);
        let names: Vec<_> = u
            .iter()
            .map(|u| (u.kind.as_str(), u.name.as_str()))
            .collect();
        assert_eq!(names, [("function", "f"), ("class", "C"), ("method", "m")]);
        assert!(u[0].callees.contains("g"));
        // docstring and annotations are dropped
        assert!(!u[0].tokens.iter().any(|t| t == "STR"));
    }

    #[test]
    fn typescript_arrow_and_class() {
        let src =
            "export const f = (a: number): number => g(a) + 1;\nexport class K { m(): void {} }\n";
        let u = extract_units("x.ts", Lang::TypeScript, src);
        let names: Vec<_> = u
            .iter()
            .map(|u| (u.kind.as_str(), u.name.as_str()))
            .collect();
        assert_eq!(names, [("function", "f"), ("class", "K"), ("method", "m")]);
    }

    #[test]
    fn csharp_skips_interfaces_and_keeps_logic_properties() {
        let src = "interface I { int Foo(int x); }\nenum E { A, B }\nclass P { public int X { get { return Compute(); } } }\nclass Q { public int X { get; set; } }\n";
        let u = extract_units("p.cs", Lang::CSharp, src);
        let n: Vec<_> = u.iter().map(|x| (x.name.as_str(), x.boilerplate)).collect();
        assert_eq!(n, [("P", false), ("Q", true)]);
    }

    #[test]
    fn csharp_classes_methods_and_normalization() {
        let src = "namespace N {\n  public class Calc {\n    public int Add(int a, int b) {\n      Console.WriteLine(\"x\");\n      return Helper(a) + b;\n    }\n    public Calc() { }\n  }\n  public record Dto(int Id);\n}\n";
        let u = extract_units("Calc.cs", Lang::CSharp, src);
        let names: Vec<_> = u
            .iter()
            .map(|x| (x.kind.as_str(), x.name.as_str()))
            .collect();
        assert_eq!(
            names,
            [
                ("class", "Calc"),
                ("method", "Add"),
                ("method", "constructor"),
                ("class", "Dto")
            ]
        );
        let add = &u[1];
        assert!(add.callees.contains("helper") && !add.callees.contains("writeline"));
        let renamed = "class Other {\n int Plus(int x, int y) {\n return Foo(x) + y;\n }\n}\n";
        let b = extract_units("o.cs", Lang::CSharp, renamed);
        assert_eq!(add.tokens, b[1].tokens);
        assert!(u[2].boilerplate);
    }

    fn toks(src: &str) -> Vec<String> {
        extract_units("x.py", Lang::Python, src).remove(0).tokens
    }

    #[test]
    fn debug_statements_are_dropped() {
        let plain = "def f(x):\n    y = g(x)\n    return y + 1\n";
        let noisy = "def f(x):\n    print(x)\n    logger.info('a')\n    y = g(x)\n    self.log.debug(y)\n    return y + 1\n";
        assert_eq!(toks(plain), toks(noisy));
        let u = extract_units("x.py", Lang::Python, noisy).remove(0);
        assert!(!u.callees.contains("print") && !u.callees.contains("info"));
        assert_eq!(
            extract_units(
                "x.ts",
                Lang::TypeScript,
                "function f(x: number) { console.log(x); return g(x) + 1; }"
            )[0]
            .tokens,
            extract_units(
                "x.ts",
                Lang::TypeScript,
                "function f(x: number) { return g(x) + 1; }"
            )[0]
            .tokens
        );
    }

    #[test]
    fn comprehension_equals_loop() {
        let loop_form = "def f(xs):\n    out = []\n    for x in xs:\n        if x > 0:\n            out.append(x * 2)\n    return out\n";
        let comp = "def f(xs):\n    out = [x * 2 for x in xs if x > 0]\n    return out\n";
        let comp_ret = "def f(xs):\n    return [x * 2 for x in xs if x > 0]\n";
        assert_eq!(toks(loop_form), toks(comp));
        assert_eq!(toks(loop_form), toks(comp_ret));
        // statement-level view agrees too
        let a = extract_units("a.py", Lang::Python, loop_form).remove(0);
        let b = extract_units("b.py", Lang::Python, comp_ret).remove(0);
        assert_eq!(a.stmts_exact, b.stmts_exact);
        assert_eq!(a.shape_seq, b.shape_seq);
    }

    #[test]
    fn augmented_assignment_and_temp_return() {
        assert_eq!(
            toks("def f(x):\n    x += 1\n    y = g(x)\n    return y\n"),
            toks("def f(x):\n    x = x + 1\n    y = g(x)\n    return y\n")
        );
        assert_eq!(
            toks("def f(a, b):\n    r = a + b\n    return r\n"),
            toks("def f(a, b):\n    return a + b\n")
        );
        let ts1 = extract_units(
            "a.ts",
            Lang::TypeScript,
            "function f(a: number) { const r = a * 2; return r; }",
        );
        let ts2 = extract_units(
            "b.ts",
            Lang::TypeScript,
            "function f(a: number) { return a * 2; }",
        );
        assert_eq!(ts1[0].tokens, ts2[0].tokens);
        assert_eq!(ts1[0].shape_seq, ts2[0].shape_seq);
    }

    #[test]
    fn field_only_classes_are_boilerplate_and_tests_are_flagged() {
        let dto = "class Item(BaseModel):\n    id: int\n    name: str\n    price: float = 0.0\n";
        let u = extract_units("x.py", Lang::Python, dto);
        assert!(u[0].boilerplate);
        let logic = "class Item:\n    id: int\n\n    def total(self):\n        return self.id\n";
        assert!(!extract_units("x.py", Lang::Python, logic)[0].boilerplate);
        let t = extract_units(
            "tests/test_a.py",
            Lang::Python,
            "def helper():\n    return 1\n",
        );
        assert!(t[0].is_test);
        assert!(
            !extract_units("src/a.py", Lang::Python, "def helper():\n    return 1\n")[0].is_test
        );
    }

    #[test]
    fn renaming_does_not_change_tokens() {
        let a = extract_units("a.py", Lang::Python, "def f(x):\n    return x + 1\n");
        let b = extract_units(
            "b.py",
            Lang::Python,
            "def other(value):\n    return value + 2\n",
        );
        assert_eq!(a[0].tokens, b[0].tokens);
    }
}
