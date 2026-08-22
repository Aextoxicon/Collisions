use streaming_iterator::StreamingIterator;
use tree_sitter::{Language, Parser, QueryCursor};
mod lang;
uniffi::setup_scaffolding!();

// 调试日志宏
macro_rules! debug_log {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        eprintln!($($arg)*);
    };
}

//UniFFI types
#[derive(uniffi::Enum, Debug, Clone)]
pub enum HighlightTokenKind {
    Keyword,
    StringLiteral,
    Comment,
    Function,
    FunctionBuiltin,
    FunctionMethod,
    Type,
    Number,
    Operator,
    Identifier,
    Variable,
    Property,
    Punctuation,
    Escape,
    ConstantBuiltin,
    Label,
    Namespace,
    Unknown,
}

impl HighlightTokenKind {
    fn from_str(s: &str) -> Self {
        match s {
            "keyword" => HighlightTokenKind::Keyword,
            "type" | "type.builtin" => HighlightTokenKind::Type,
            "string" | "string.special" => HighlightTokenKind::StringLiteral,
            "comment" => HighlightTokenKind::Comment,
            "function" => HighlightTokenKind::Function,
            "function.builtin" => HighlightTokenKind::FunctionBuiltin,
            "function.method" => HighlightTokenKind::FunctionMethod,
            "number" => HighlightTokenKind::Number,
            "operator" => HighlightTokenKind::Operator,
            "label" => HighlightTokenKind::Label,
            "namespace" => HighlightTokenKind::Namespace,
            "identifier" => HighlightTokenKind::Identifier,
            "variable" => HighlightTokenKind::Variable,
            "property" => HighlightTokenKind::Property,
            "punctuation" => HighlightTokenKind::Punctuation,
            "escape" => HighlightTokenKind::Escape,
            "constant.builtin" => HighlightTokenKind::ConstantBuiltin,
            _ => HighlightTokenKind::Unknown,
        }
    }
}

#[derive(uniffi::Record)]
pub struct HighlightToken {
    pub start_byte: u64,
    pub end_byte: u64,
    pub kind: HighlightTokenKind,
}

#[derive(uniffi::Record)]
pub struct OutlineNode {
    pub kind: String,
    pub name: String,
    pub detail: String,
    pub start_byte: u64,
    pub end_byte: u64,
    pub children: Vec<OutlineNode>,
}

#[derive(uniffi::Record)]
pub struct CodeParseResult {
    // 按需返回每行的高亮 token
    pub highlights_by_line: Vec<Vec<HighlightToken>>,
    pub outline: Vec<OutlineNode>,
}

struct ScanSource {
    line_boundaries: Vec<(u64, u64)>,
    /// UTF-8 字节偏移转UTF-16偏移的映射表，若源文件纯ASCII则为None
    byte_to_utf16_map: Option<Vec<u32>>,
}

fn scan_source(source: &str) -> ScanSource {
    // 纯ASCII-字节偏移=UTF-16 偏移
    if source.is_ascii() {
        let mut boundaries = Vec::new();
        let mut line_start: u64 = 0;
        for (i, &b) in source.as_bytes().iter().enumerate() {
            if b == b'\n' {
                let pos = (i + 1) as u64;
                boundaries.push((line_start, pos));
                line_start = pos;
            }
        }
        let len = source.len() as u64;
        if line_start < len || (line_start == len && line_start > 0) {
            boundaries.push((line_start, len));
        }
        return ScanSource {
            line_boundaries: boundaries,
            byte_to_utf16_map: None,
        };
    }

    // 非ASCII单次遍历同时构建映射表+行边界
    let n = source.len();
    let mut map = Vec::with_capacity(n + 1);
    let mut boundaries = Vec::new();
    let mut line_start: u64 = 0;
    let mut utf16_pos: u64 = 0;
    for ch in source.chars() {
        let utf16_len = ch.len_utf16() as u64;
        for _ in 0..ch.len_utf8() {
            map.push(utf16_pos as u32);
        }
        utf16_pos += utf16_len;
        if ch == '\n' {
            boundaries.push((line_start, utf16_pos));
            line_start = utf16_pos;
        }
    }
    map.push(utf16_pos as u32);
    if line_start < utf16_pos || (line_start == utf16_pos && line_start > 0) {
        boundaries.push((line_start, utf16_pos));
    }
    ScanSource {
        line_boundaries: boundaries,
        byte_to_utf16_map: Some(map),
    }
}

fn map_byte(map: &[u32], byte_pos: u64) -> u64 {
    map.get(byte_pos as usize).copied().unwrap_or(0) as u64
}

fn convert_highlights(map: Option<&[u32]>, highlights: &mut [HighlightToken]) {
    let Some(map) = map else { return };
    for h in highlights {
        h.start_byte = map_byte(map, h.start_byte);
        h.end_byte = map_byte(map, h.end_byte);
    }
}

fn convert_outline(map: Option<&[u32]>, outline: &mut [OutlineNode]) {
    let Some(map) = map else { return };
    for node in outline {
        node.start_byte = map_byte(map, node.start_byte);
        node.end_byte = map_byte(map, node.end_byte);
        convert_outline(Some(map), &mut node.children);
    }
}

fn split_highlights_by_line(
    line_boundaries: &[(u64, u64)],
    highlights: &[HighlightToken],
) -> Vec<Vec<HighlightToken>> {
    let line_count = line_boundaries.len();
    if line_count == 0 {
        return Vec::new();
    }

    // 预先分配每行的 Vec
    let mut result: Vec<Vec<HighlightToken>> = (0..line_count).map(|_| Vec::new()).collect();
    let line_starts: Vec<u64> = line_boundaries.iter().map(|(s, _)| *s).collect();
    for h in highlights {
        let start_line = match line_starts.binary_search(&h.start_byte) {
            Ok(idx) => idx,
            Err(idx) => {
                if idx == 0 {
                    continue; // 在文件开头之前，不应发生
                }
                idx - 1
            }
        };
        let end_line = {
            let idx = match line_starts.binary_search(&h.end_byte) {
                Ok(idx) | Err(idx) => idx,
            };
            if idx == 0 {
                continue;
            }
            idx - 1
        };
        for line_idx in start_line..=end_line.min(line_count - 1) {
            let (line_start, line_end) = line_boundaries[line_idx];
            let overlap_start = h.start_byte.max(line_start);
            let overlap_end = h.end_byte.min(line_end);
            if overlap_start < overlap_end {
                let line_len = line_end.saturating_sub(line_start);
                result[line_idx].push(HighlightToken {
                    start_byte: overlap_start.saturating_sub(line_start).min(line_len),
                    end_byte: overlap_end.saturating_sub(line_start).min(line_len),
                    kind: h.kind.clone(),
                });
            }
        }
    }
    // 排序
    for tokens in &mut result {
        tokens.sort_by_key(|t| t.start_byte);
    }

    result
}

//helpers
fn extract_name(node: tree_sitter::Node, source: &[u8]) -> String {
    if let Some(name_node) = node.child_by_field_name("name") {
        name_node.utf8_text(source).unwrap_or("").to_string()
    } else {
        String::new()
    }
}

// 只有这些节点类型会生成 OutlineNode
const OUTLINE_STRUCTURAL_KINDS: &[&str] = &[

    "function_definition",
    "function_declaration",
    "function_item",
    "method_definition",
    "method_declaration",
    "generator_function_declaration",

    "class_definition",
    "class_declaration",
    "class_specifier",
    "struct_declaration",
    "struct_specifier",
    "struct_item",
    "interface_declaration",
    "type_alias_declaration",
    "type_item",
    "record_declaration",
    "enum_declaration",
    "enum_specifier",
    "enum_item",
    "trait_item",
    "impl_item",
    "annotation_type_declaration",

    "namespace_definition",
    "namespace_declaration",
    "mod_item",

    "static_item",
    "const_item",
    "type_declaration",
    "package_clause",
];

/// 最大 outline 嵌套深度
const MAX_OUTLINE_DEPTH: usize = 16;
/// 最大 outline 节点总数（超出直接截断）
const MAX_OUTLINE_NODES: usize = 1000;

fn is_structural_kind(kind: &str) -> bool {
    OUTLINE_STRUCTURAL_KINDS.contains(&kind)
}

/// 结构性节点：创建 OutlineNode 并递归收集子节点
/// 非结构性节点：不创建节点，但继续深入子节点
fn collect_outline(
    node: tree_sitter::Node,
    source: &[u8],
    depth: usize,
    counter: &mut usize,
    out: &mut Vec<OutlineNode>,
) {
    if depth > MAX_OUTLINE_DEPTH || *counter >= MAX_OUTLINE_NODES {
        return;
    }

    if is_structural_kind(node.kind()) {
        *counter += 1;

        let mut children = Vec::new();
        collect_outline_children(node, source, depth + 1, counter, &mut children);

        out.push(OutlineNode {
            kind: node.kind().to_string(),
            name: extract_name(node, source),
            detail: String::new(),
            start_byte: node.start_byte() as u64,
            end_byte: node.end_byte() as u64,
            children,
        });
    } else {
        collect_outline_children(node, source, depth, counter, out);
    }
}

fn collect_outline_children(
    node: tree_sitter::Node,
    source: &[u8],
    depth: usize,
    counter: &mut usize,
    out: &mut Vec<OutlineNode>,
) {
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            let child = cursor.node();
            if child.is_named() {
                collect_outline(child, source, depth, counter, out);
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

//exported
#[uniffi::export]
pub fn parse_code(source: String, extension: String) -> CodeParseResult {
    let source_bytes = source.as_bytes();
    // 找grammar
    let grammar = match lang::get_grammar(&extension) {
        Some(g) => g,
        None => {
            debug_log!("[RUST] unsupported extension: {}", extension);
            return CodeParseResult {
                highlights_by_line: Vec::new(),
                outline: Vec::new(),
            };
        }
    };
    let language: Language = grammar.language.clone();

    let mut parser = Parser::new();
    if parser.set_language(&language).is_err() {
        debug_log!("[RUST] failed to set language for extension: {}", extension);
        return CodeParseResult {
            highlights_by_line: Vec::new(),
            outline: Vec::new(),
        };
    }

    let tree = match parser.parse(source_bytes, None) {
        Some(t) => t,
        None => {
            debug_log!("[RUST] failed to parse source for extension: {}", extension);
            return CodeParseResult {
                highlights_by_line: Vec::new(),
                outline: Vec::new(),
            };
        }
    };

    let query = &grammar.compiled_query;

    debug_log!(
        "[RUST] parse_code called, source length={}, extension={}",
        source.len(),
        extension
    );
    debug_log!("[RUST] query capture names: {:?}", query.capture_names());

    let mut highlights = {
        let mut qc = QueryCursor::new();
        let mut results = Vec::new();
        let mut matches = qc.matches(query, tree.root_node(), source.as_bytes());
        while let Some(match_) = matches.next() {
            for capture in match_.captures {
                let node = capture.node;
                let kind_str = query.capture_names()[capture.index as usize];
                results.push(HighlightToken {
                    start_byte: node.start_byte() as u64,
                    end_byte: node.end_byte() as u64,
                    kind: HighlightTokenKind::from_str(kind_str),
                });
            }
        }
        // 线性去重
        results.sort_by_key(|t| (t.start_byte, t.end_byte));
        results.dedup_by_key(|t| (t.start_byte, t.end_byte));
        debug_log!("[RUST] highlights count: {} (deduplicated)", results.len());
        for h in &results {
            debug_log!(
                "[RUST]   highlight: start={} end={} kind={:?}",
                h.start_byte, h.end_byte, h.kind
            );
        }
        results
    };

    let outline = {
        let root = tree.root_node();
        let mut top = Vec::new();
        let mut counter = 0usize;
        collect_outline_children(root, source_bytes, 0, &mut counter, &mut top);
        debug_log!(
            "[RUST] outline count: {} (max depth: {}, max nodes: {})",
            top.len(),
            MAX_OUTLINE_DEPTH,
            MAX_OUTLINE_NODES
        );
        top
    };

    let mut result = CodeParseResult {
        highlights_by_line: Vec::new(),
        outline,
    };

    // 获取 UTF-16 映射表和行边界
    let scan = scan_source(&source);
    convert_highlights(scan.byte_to_utf16_map.as_deref(), &mut highlights);
    convert_outline(scan.byte_to_utf16_map.as_deref(), &mut result.outline);

    result.highlights_by_line = split_highlights_by_line(&scan.line_boundaries, &highlights);

    debug_log!(
        "[RUST] returning result with {} lines of highlights",
        result.highlights_by_line.len()
    );
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_queries_compile() {
        // 验证每个 grammar 的 highlight query 都能成功编译
        let extensions = [".c", ".h", ".cpp", ".hpp", ".go", ".py", ".js", ".mjs", ".cjs", ".ts", ".tsx", ".sh", ".bash", ".zsh", ".cs", ".java", ".json", ".css", ".rs"];
        let mut failures = Vec::new();
        for ext in extensions {
            let grammar =
                crate::lang::get_grammar(ext).unwrap_or_else(|| panic!("no grammar for {}", ext));
            let names = grammar.compiled_query.capture_names();
            eprintln!("[QUERY OK] {} ({} captures)", ext, names.len());
            if names.is_empty() {
                failures.push(format!("{}: no captures", ext));
            }
        }
        assert!(failures.is_empty(), "query failures:\n{}", failures.join("\n"));
    }

    #[test]
    fn test_parse_go_code() {
        let source = r#"package main

import "fmt"

func main() {
    fmt.Println("hello world")
}
"#;
        let result = parse_code(source.to_string(), ".go".to_string());
        eprintln!("=== HIGHLIGHTS BY LINE ===");
        for (i, line_tokens) in result.highlights_by_line.iter().enumerate() {
            eprintln!("  line {}: {} tokens", i, line_tokens.len());
            for h in line_tokens {
                eprintln!("    [{:?}] offset {}-{}", h.kind, h.start_byte, h.end_byte);
            }
        }
        eprintln!("=== OUTLINE ===");
        fn print_outline(nodes: &[OutlineNode], depth: usize) {
            let indent = "  ".repeat(depth);
            for node in nodes {
                eprintln!("{}{} name={:?} bytes {}-{} children={}", indent, node.kind, node.name, node.start_byte, node.end_byte, node.children.len());
                print_outline(&node.children, depth + 1);
            }
        }
        print_outline(&result.outline, 0);
        assert!(!result.highlights_by_line.is_empty(), "expected some highlights");
    }

    #[test]
    fn test_all_languages_parse_and_highlight() {
        // 覆盖所有已注册的扩展名，确保每个 grammar 都能 parse + query
        let cases = [
            (".c", "#include <stdio.h>\nint main() { /* c comment */ int x = 1; return 0; }\n"),
            (".h", "#ifndef H\n#define H\nint add(int a, int b); // header comment\n#endif\n"),
            (".cpp", "int main() { // cpp comment\n  auto x = 1; /* block */ return x;\n}\n"),
            (".hpp", "class Foo { public: int bar(); }; // hpp comment\n"),
            (".go", "package main\n\n// go comment\nfunc main() {\n\tfmt.Println(\"hi\")\n}\n"),
            (".py", "# python comment\n\ndef main():  # inline\n    return 1\n"),
            (".js", "// js comment\nfunction foo() { /* block */ return 1; }\n"),
            (".mjs", "// mjs comment\nexport const x = 1;\n"),
            (".cjs", "// cjs comment\nmodule.exports = 1;\n"),
            (".ts", "// ts comment\nfunction add(a: number): number { return a; }\n"),
            (".tsx", "// tsx comment\nconst el = <div>hi</div>;\n"),
            (".sh", "#!/bin/bash\n# shell comment\necho hello\n"),
            (".bash", "# bash comment\necho hi\n"),
            (".zsh", "# zsh comment\necho hi\n"),
            (".cs", "// cs comment\nclass C { int M() { return 1; } }\n"),
            (".java", "// java comment\npublic class Main { public static void main(String[] a) { /* block */ } }\n"),
            (".json", "{\n  \"key\": \"value\"  // jsonc not std, just test\n}\n"),
            (".css", "/* css comment */\nbody { color: red; }\n"),
            (".rs", "// rust comment\nfn main() { /* block */ let x = 1; }\n"),
        ];

        for (ext, source) in cases {
            let result = parse_code(source.to_string(), ext.to_string());
            eprintln!("=== {} ===", ext);
            eprintln!("  lines: {} (source has {} lines)", result.highlights_by_line.len(), source.lines().count());
            let total_tokens: usize = result.highlights_by_line.iter().map(|t| t.len()).sum();
            eprintln!("  total tokens: {}", total_tokens);

            // 简单断言：至少有一个 token（C 语言至少能识别 #include 或 comment）
            // 注意：JSON 标准语法里 // 不是注释，只要没 panic 就算通过
            assert!(
                total_tokens > 0,
                "{}: expected at least 1 highlight token, got 0",
                ext
            );

            // 打印前几行 token 种类用于调试
            for (i, line) in result.highlights_by_line.iter().enumerate().take(3) {
                let kinds: Vec<String> = line.iter().map(|t| format!("{:?}", t.kind)).collect();
                eprintln!("  line {}: {}", i, kinds.join(", "));
            }
        }
    }

    #[test]
    fn test_parse_python_code() {
        let source = r#"import os
import sys

def main():
    print("hello world")
    x = 42
    return x
"#;
        let result = parse_code(source.to_string(), ".py".to_string());
        eprintln!("=== PYTHON HIGHLIGHTS BY LINE ===");
        for (i, line_tokens) in result.highlights_by_line.iter().enumerate() {
            eprintln!("  line {}: {} tokens", i, line_tokens.len());
            for h in line_tokens {
                eprintln!("    [{:?}] offset {}-{}", h.kind, h.start_byte, h.end_byte);
            }
        }
        assert!(!result.highlights_by_line.is_empty(), "expected some highlights");
        // 至少应该有 keyword (import, def), string, identifier 等
        let total_tokens: usize = result.highlights_by_line.iter().map(|t| t.len()).sum();
        assert!(total_tokens > 5, "expected more than 5 tokens, got {}", total_tokens);
    }
}
