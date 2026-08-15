use tree_sitter::{Language, Parser, Query, QueryCursor, StreamingIterator};
mod lang;
mod ffi;
uniffi::setup_scaffolding!();

// 调试日志宏
macro_rules! debug_log {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        eprintln!($($arg)*);
    };
}

//UniFFI types
#[derive(uniffi::Enum, Debug, Clone, serde::Serialize)]
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

#[derive(uniffi::Record, serde::Serialize)]
pub struct HighlightToken {
    pub start_byte: u64,
    pub end_byte: u64,
    pub kind: HighlightTokenKind,
}

#[derive(uniffi::Record, serde::Serialize)]
pub struct OutlineNode {
    pub kind: String,
    pub name: String,
    pub detail: String,
    pub start_byte: u64,
    pub end_byte: u64,
    pub children: Vec<OutlineNode>,
}

#[derive(uniffi::Record, serde::Serialize)]
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

/// 单次遍历源文件，同时构建行边界和 UTF-16 映射表
fn scan_source(source: &str) -> ScanSource {
    // 快速路径：纯 ASCII — 字节偏移 == UTF-16 偏移，无需映射表
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
    // 构建行起始偏移数组用于二分查找
    let line_starts: Vec<u64> = line_boundaries.iter().map(|(s, _)| *s).collect();
    for h in highlights {
        // 找到的行号 -1 就是 h 所在的行
        let line_idx = match line_starts.binary_search(&h.start_byte) {
            Ok(idx) => idx,
            Err(idx) => {
                // idx 是第一个 > h.start_byte 的元素
                if idx == 0 {
                    continue; // 在文件开头之前，不应发生
                }
                idx - 1
            }
        };
        let (line_start, line_end) = line_boundaries[line_idx];
        // 检查 token 是否真的与该行重叠
        if h.start_byte < line_end && h.end_byte > line_start {
            let line_len = line_end.saturating_sub(line_start);
            result[line_idx].push(HighlightToken {
                start_byte: h.start_byte.saturating_sub(line_start).min(line_len),
                end_byte: h.end_byte.saturating_sub(line_start).min(line_len),
                kind: h.kind.clone(),
            });
        }
    }
    // 对每行内的 token 按 start_byte 排序，并过滤掉空 token
    for tokens in &mut result {
        tokens.retain(|t| t.end_byte > 0);
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

fn extract_detail(_node: tree_sitter::Node, _source: &[u8]) -> String {
    String::new()
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
        if *counter >= MAX_OUTLINE_NODES {
            return;
        }
        *counter += 1;

        let mut children = Vec::new();
        collect_outline_children(node, source, depth + 1, counter, &mut children);

        out.push(OutlineNode {
            kind: node.kind().to_string(),
            name: extract_name(node, source),
            detail: extract_detail(node, source),
            start_byte: node.start_byte() as u64,
            end_byte: node.end_byte() as u64,
            children,
        });
    } else {
        // 非结构性节点上浮
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
    let grammar = lang::get_grammar(&extension)
        .unwrap_or_else(|| panic!("Unsupported file extension: {}", extension));
    let language: Language = grammar.language.clone();

    let mut parser = Parser::new();
    parser
        .set_language(&language)
        .expect("Failed to set language");

    let tree = parser
        .parse(source_bytes, None)
        .expect("Failed to parse source");

    let query =
        Query::new(&language, grammar.highlight_query).expect("Invalid highlight query");

    debug_log!(
        "[RUST] parse_code called, source length={}, extension={}",
        source.len(),
        extension
    );
    debug_log!("[RUST] query capture names: {:?}", query.capture_names());

    let mut highlights = {
        let mut qc = QueryCursor::new();
        let mut results = Vec::new();
        let mut matches = qc.matches(&query, tree.root_node(), source.as_bytes());
        while let Some(match_) = matches.next() {
            for capture in match_.captures {
                let node = capture.node;
                let kind_str = query.capture_names()[capture.index as usize];
                results.push(HighlightToken {
                    start_byte: node.start_byte() as u64,
                    end_byte: node.end_byte() as u64,
                    kind: HighlightTokenKind::from_str(&kind_str),
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

    #[test]
    fn test_parse_csharp_code() {
        let source = r#"using System;

namespace HelloWorld
{
    class Program
    {
        static void Main(string[] args)
        {
            Console.WriteLine("Hello World");
        }
    }
}
"#;
        let result = parse_code(source.to_string(), ".cs".to_string());
        eprintln!("=== CSHARP HIGHLIGHTS BY LINE ===");
        for (i, line_tokens) in result.highlights_by_line.iter().enumerate() {
            eprintln!("  line {}: {} tokens", i, line_tokens.len());
            for h in line_tokens {
                eprintln!("    [{:?}] offset {}-{}", h.kind, h.start_byte, h.end_byte);
            }
        }
        eprintln!("=== CSHARP OUTLINE ===");
        fn print_outline(nodes: &[OutlineNode], depth: usize) {
            let indent = "  ".repeat(depth);
            for node in nodes {
                eprintln!("{}{} name={:?} bytes {}-{} children={}", indent, node.kind, node.name, node.start_byte, node.end_byte, node.children.len());
                print_outline(&node.children, depth + 1);
            }
        }
        print_outline(&result.outline, 0);
        assert!(!result.highlights_by_line.is_empty(), "expected some highlights");
        // 至少应该有 keyword (using, namespace, class, static, void, string), type, function 等
        let total_tokens: usize = result.highlights_by_line.iter().map(|t| t.len()).sum();
        assert!(total_tokens > 5, "expected more than 5 tokens, got {}", total_tokens);
    }
}