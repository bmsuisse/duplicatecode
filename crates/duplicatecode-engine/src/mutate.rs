//! Semantics-agnostic source mutations used to test detector robustness. Each mutation rewrites
//! source text the way a different author (or a lazy refactoring) might, while keeping the
//! number and order of functions/classes unchanged.

use crate::lang::Lang;
use std::collections::{BTreeMap, BTreeSet};
use tree_sitter::{Node, Parser};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mutation {
    /// Rename function names, parameters and local variables.
    RenameAll,
    /// Swap adjacent independent statements.
    SwapStatements,
    /// `return expr` -> `tmp = expr; return tmp`.
    TempVariable,
    /// Insert debug logging statements.
    AddLogging,
    /// Insert an unused local variable.
    AddDeadCode,
    /// All of the above, applied in order.
    Combined,
}

impl Mutation {
    pub const ALL: [Mutation; 6] = [
        Mutation::RenameAll,
        Mutation::SwapStatements,
        Mutation::TempVariable,
        Mutation::AddLogging,
        Mutation::AddDeadCode,
        Mutation::Combined,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Mutation::RenameAll => "rename-all",
            Mutation::SwapStatements => "swap-statements",
            Mutation::TempVariable => "temp-variable",
            Mutation::AddLogging => "add-logging",
            Mutation::AddDeadCode => "add-dead-code",
            Mutation::Combined => "combined",
        }
    }
}

struct Edit {
    start: usize,
    end: usize,
    text: String,
}

pub fn apply(m: Mutation, lang: Lang, source: &str) -> String {
    match m {
        Mutation::RenameAll => apply_one(rename_edits, lang, source),
        Mutation::SwapStatements => apply_one(swap_edits, lang, source),
        Mutation::TempVariable => apply_one(temp_edits, lang, source),
        Mutation::AddLogging => apply_one(logging_edits, lang, source),
        Mutation::AddDeadCode => apply_one(dead_code_edits, lang, source),
        Mutation::Combined => {
            let mut s = source.to_string();
            for step in [rename_edits, swap_edits, temp_edits, logging_edits, dead_code_edits] {
                s = apply_one(step, lang, &s);
            }
            s
        }
    }
}

type EditFn = fn(Lang, Node, &str) -> Vec<Edit>;

fn apply_one(f: EditFn, lang: Lang, source: &str) -> String {
    let mut parser = Parser::new();
    if parser.set_language(&lang.ts_language()).is_err() {
        return source.to_string();
    }
    let Some(tree) = parser.parse(source, None) else {
        return source.to_string();
    };
    let mut edits = f(lang, tree.root_node(), source);
    edits.sort_by_key(|e| std::cmp::Reverse(e.start));
    let mut out = source.to_string();
    let mut last_start = usize::MAX;
    for e in edits {
        if e.end > last_start {
            continue; // overlapping edit: skip
        }
        out.replace_range(e.start..e.end, &e.text);
        last_start = e.start;
    }
    out
}

fn text<'a>(n: Node, src: &'a str) -> &'a str {
    &src[n.start_byte()..n.end_byte()]
}

fn walk<'t>(node: Node<'t>, f: &mut dyn FnMut(Node<'t>)) {
    f(node);
    for i in 0..node.child_count() {
        if let Some(c) = node.child(i) {
            walk(c, f);
        }
    }
}

fn is_function(lang: Lang, n: Node) -> bool {
    match lang {
        Lang::Python => n.kind() == "function_definition",
        _ => matches!(
            n.kind(),
            "function_declaration" | "method_definition" | "arrow_function" | "function_expression" | "function"
        ),
    }
}

/// Body blocks of all functions.
fn function_bodies<'t>(lang: Lang, root: Node<'t>) -> Vec<Node<'t>> {
    let mut out = Vec::new();
    walk(root, &mut |n| {
        if is_function(lang, n) {
            if let Some(b) = n.child_by_field_name("body") {
                if matches!(b.kind(), "block" | "statement_block") {
                    out.push(b);
                }
            }
        }
    });
    out
}

fn statements<'t>(block: Node<'t>) -> Vec<Node<'t>> {
    (0..block.named_child_count())
        .filter_map(|i| block.named_child(i as u32))
        .filter(|c| c.kind() != "comment")
        .collect()
}

fn is_docstring(lang: Lang, n: Node) -> bool {
    lang == Lang::Python
        && n.kind() == "expression_statement"
        && n.named_child_count() == 1
        && n.named_child(0).is_some_and(|c| c.kind() == "string")
}

fn indent_of(n: Node) -> String {
    " ".repeat(n.start_position().column)
}

/// True when the statement starts its own line (safe to insert a line before it).
fn starts_line(n: Node, src: &str) -> bool {
    let line_start = src[..n.start_byte()].rfind('\n').map_or(0, |i| i + 1);
    src[line_start..n.start_byte()].chars().all(|c| c == ' ' || c == '\t')
}

// ---- rename ----------------------------------------------------------------------------

const WORDS: [&str; 12] =
    ["item", "value", "entry", "current", "outcome", "buf", "data", "node", "key", "acc", "part", "cursor"];

fn collect_pattern_idents<'t>(n: Node<'t>, out: &mut Vec<Node<'t>>) {
    if n.kind() == "identifier" {
        out.push(n);
        return;
    }
    if matches!(n.kind(), "tuple_pattern" | "list_pattern" | "pattern_list" | "tuple" | "list") {
        for i in 0..n.named_child_count() {
            if let Some(c) = n.named_child(i as u32) {
                collect_pattern_idents(c, out);
            }
        }
    }
}

fn rename_edits(lang: Lang, root: Node, src: &str) -> Vec<Edit> {
    // 1. binding names per whole file (scoping is ignored on purpose: consistent renaming)
    let mut bindings: BTreeSet<String> = BTreeSet::new();
    let mut fn_names: Vec<Node> = Vec::new();
    walk(root, &mut |n| {
        let mut push = |x: Node| {
            let t = text(x, src);
            if t != "self" && t != "cls" && t != "this" {
                bindings.insert(t.to_string());
            }
        };
        let mut nodes = Vec::new();
        match (lang, n.kind()) {
            (Lang::Python, "function_definition") => {
                if let Some(name) = n.child_by_field_name("name") {
                    fn_names.push(name);
                }
                if let Some(params) = n.child_by_field_name("parameters") {
                    for i in 0..params.named_child_count() {
                        let Some(p) = params.named_child(i as u32) else { continue };
                        match p.kind() {
                            "identifier" => nodes.push(p),
                            "default_parameter" | "typed_default_parameter" => {
                                if let Some(x) = p.child_by_field_name("name") {
                                    nodes.push(x);
                                }
                            }
                            "typed_parameter" | "list_splat_pattern" | "dictionary_splat_pattern" => {
                                if let Some(x) = p.named_child(0) {
                                    if x.kind() == "identifier" {
                                        nodes.push(x);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            (Lang::Python, "assignment" | "augmented_assignment" | "for_statement" | "for_in_clause") => {
                if let Some(l) = n.child_by_field_name("left") {
                    collect_pattern_idents(l, &mut nodes);
                }
            }
            (Lang::Python, "as_pattern") => {
                if let Some(l) = n.child_by_field_name("alias") {
                    collect_pattern_idents(l, &mut nodes);
                } else if let Some(l) = n.named_child(n.named_child_count().saturating_sub(1) as u32) {
                    collect_pattern_idents(l, &mut nodes);
                }
            }
            (Lang::TypeScript | Lang::Tsx, "function_declaration") => {
                if let Some(name) = n.child_by_field_name("name") {
                    fn_names.push(name);
                }
            }
            (Lang::TypeScript | Lang::Tsx, "required_parameter" | "optional_parameter") => {
                if let Some(x) = n.child_by_field_name("pattern") {
                    if x.kind() == "identifier" {
                        nodes.push(x);
                    }
                }
            }
            (Lang::TypeScript | Lang::Tsx, "variable_declarator") => {
                if let Some(x) = n.child_by_field_name("name") {
                    if x.kind() == "identifier" {
                        nodes.push(x);
                    }
                }
            }
            (Lang::TypeScript | Lang::Tsx, "arrow_function") => {
                if let Some(x) = n.child_by_field_name("parameter") {
                    nodes.push(x);
                }
            }
            _ => {}
        }
        for x in nodes {
            push(x);
        }
    });
    let map: BTreeMap<String, String> = bindings
        .into_iter()
        .enumerate()
        .map(|(i, b)| (b, format!("{}{}", WORDS[i % WORDS.len()], i / WORDS.len())))
        .collect();

    // 2. rewrite every identifier occurrence of a binding (not attribute/keyword names)
    let mut edits = Vec::new();
    walk(root, &mut |n| {
        if n.kind() != "identifier" {
            return;
        }
        if let Some(p) = n.parent() {
            let is_field = |f: &str| p.child_by_field_name(f).is_some_and(|c| c.id() == n.id());
            if (p.kind() == "attribute" && is_field("attribute"))
                || (p.kind() == "keyword_argument" && is_field("name"))
            {
                return;
            }
        }
        if let Some(new) = map.get(text(n, src)) {
            edits.push(Edit { start: n.start_byte(), end: n.end_byte(), text: new.clone() });
        }
    });
    // 3. the units' own names
    let generic = if lang == Lang::Python { "run_task" } else { "runTask" };
    for (i, n) in fn_names.iter().enumerate() {
        if !edits.iter().any(|e| e.start == n.start_byte()) {
            edits.push(Edit { start: n.start_byte(), end: n.end_byte(), text: format!("{generic}_{i}") });
        }
    }
    edits
}

// ---- swap -------------------------------------------------------------------------------

fn idents_of(n: Node, src: &str) -> BTreeSet<String> {
    let mut s = BTreeSet::new();
    walk(n, &mut |x| {
        if matches!(x.kind(), "identifier" | "property_identifier") {
            s.insert(text(x, src).to_string());
        }
    });
    s
}

fn is_simple(lang: Lang, n: Node) -> bool {
    match lang {
        Lang::Python => n.kind() == "expression_statement",
        _ => matches!(n.kind(), "expression_statement" | "lexical_declaration"),
    }
}

fn swap_edits(lang: Lang, root: Node, src: &str) -> Vec<Edit> {
    let mut edits = Vec::new();
    for body in function_bodies(lang, root) {
        let st = statements(body);
        let mut i = 0;
        while i + 1 < st.len() {
            let (a, b) = (st[i], st[i + 1]);
            if is_simple(lang, a)
                && is_simple(lang, b)
                && !is_docstring(lang, a)
                && idents_of(a, src).is_disjoint(&idents_of(b, src))
            {
                edits.push(Edit { start: a.start_byte(), end: a.end_byte(), text: text(b, src).to_string() });
                edits.push(Edit { start: b.start_byte(), end: b.end_byte(), text: text(a, src).to_string() });
                i += 2;
            } else {
                i += 1;
            }
        }
    }
    edits
}

// ---- temp variable ------------------------------------------------------------------------

fn temp_edits(lang: Lang, root: Node, src: &str) -> Vec<Edit> {
    let mut edits = Vec::new();
    walk(root, &mut |n| {
        if n.kind() != "return_statement" || !starts_line(n, src) {
            return;
        }
        let Some(expr) = n.named_child(0) else { return };
        if matches!(expr.kind(), "identifier" | "true" | "false" | "none" | "null") {
            return;
        }
        let ind = indent_of(n);
        let e = text(expr, src);
        let (tmp, semi) = if lang == Lang::Python { ("result_value", "") } else { ("resultValue", ";") };
        let decl = if lang == Lang::Python { "" } else { "const " };
        edits.push(Edit {
            start: n.start_byte(),
            end: n.end_byte(),
            text: format!("{decl}{tmp} = {e}{semi}\n{ind}return {tmp}{semi}"),
        });
    });
    edits
}

// ---- noise ----------------------------------------------------------------------------------

fn insert_before(lang: Lang, n: Node, src: &str, stmt: &str) -> Option<Edit> {
    if !starts_line(n, src) {
        return None;
    }
    let _ = lang;
    Some(Edit { start: n.start_byte(), end: n.start_byte(), text: format!("{stmt}\n{}", indent_of(n)) })
}

fn logging_edits(lang: Lang, root: Node, src: &str) -> Vec<Edit> {
    let stmt = if lang == Lang::Python { "print(\"debug\")" } else { "console.log(\"debug\");" };
    let mut edits = Vec::new();
    for body in function_bodies(lang, root) {
        if let Some(first) = statements(body).into_iter().find(|s| !is_docstring(lang, *s)) {
            edits.extend(insert_before(lang, first, src, stmt));
        }
    }
    walk(root, &mut |n| {
        if n.kind() == "return_statement" {
            edits.extend(insert_before(lang, n, src, stmt));
        }
    });
    edits
}

fn dead_code_edits(lang: Lang, root: Node, src: &str) -> Vec<Edit> {
    let stmt = if lang == Lang::Python { "unused_marker = 42" } else { "const unusedMarker = 42;" };
    let mut edits = Vec::new();
    for body in function_bodies(lang, root) {
        if let Some(first) = statements(body).into_iter().find(|s| !is_docstring(lang, *s)) {
            edits.extend(insert_before(lang, first, src, stmt));
        }
    }
    edits
}

#[cfg(test)]
mod tests {
    use super::*;

    const PY: &str = "def total(items, factor):\n    \"\"\"Sum.\"\"\"\n    acc = 0\n    count = 1\n    for item in items:\n        acc += item * factor\n    return acc + count\n";

    #[test]
    fn python_mutations_stay_parseable_and_keep_unit_count() {
        for m in Mutation::ALL {
            let out = apply(m, Lang::Python, PY);
            let units = crate::extract_units("x.py", Lang::Python, &out);
            assert_eq!(units.len(), 1, "{}: {out}", m.name());
            // no parse errors
            let mut p = Parser::new();
            p.set_language(&Lang::Python.ts_language()).unwrap();
            assert!(!p.parse(&out, None).unwrap().root_node().has_error(), "{}: {out}", m.name());
        }
    }

    #[test]
    fn rename_changes_names_but_not_structure() {
        let out = apply(Mutation::RenameAll, Lang::Python, PY);
        assert!(!out.contains("items") && !out.contains("total"));
        let a = crate::extract_units("a.py", Lang::Python, PY);
        let b = crate::extract_units("b.py", Lang::Python, &out);
        assert_eq!(a[0].tokens, b[0].tokens);
    }

    #[test]
    fn temp_variable_and_swap() {
        let out = apply(Mutation::TempVariable, Lang::Python, PY);
        assert!(out.contains("result_value = acc + count\n    return result_value"));
        let out = apply(Mutation::SwapStatements, Lang::Python, PY);
        assert!(out.contains("count = 1\n    acc = 0"));
    }

    #[test]
    fn typescript_mutations_parse() {
        let ts = "export function add(a: number, b: number): number {\n  const x = a + 1;\n  const y = b + 2;\n  return x * y;\n}\n";
        for m in Mutation::ALL {
            let out = apply(m, Lang::TypeScript, ts);
            let mut p = Parser::new();
            p.set_language(&Lang::TypeScript.ts_language()).unwrap();
            assert!(!p.parse(&out, None).unwrap().root_node().has_error(), "{}: {out}", m.name());
        }
    }
}
