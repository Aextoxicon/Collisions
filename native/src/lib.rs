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

// 构建 UTF-8 字节偏移 → UTF-16 代码单元偏移的映射表
fn build_byte_to_utf16_map(source: &str) -> Option<Vec<u32>> {
    let n = source.len();
    if n == 0 {
        return Some(vec![0]);
    }
    // 纯ASCII文件：byte offset == UTF-16 code unit offset
    if source.is_ascii() {
        return None;
    }
    let mut map = Vec::with_capacity(n + 1);
    let mut utf16_pos: u32 = 0;
    for ch in source.chars() {
        let utf16_len = ch.len_utf16() as u32;
        for _ in 0..ch.len_utf8() {
            map.push(utf16_pos);
        }
        utf16_pos += utf16_len;
    }
    map.push(utf16_pos); // 文件尾位置
    Some(map)
}

fn map_byte(map: &[u32], byte_pos: u64) -> u64 {
    map.get(byte_pos as usize).copied().unwrap_or(0) as u64
}

fn convert_highlights(map: &[u32], highlights: &mut [HighlightToken]) {
    for h in highlights {
        h.start_byte = map_byte(map, h.start_byte);
        h.end_byte = map_byte(map, h.end_byte);
    }
}

fn convert_outline(map: &[u32], outline: &mut [OutlineNode]) {
    for node in outline {
        node.start_byte = map_byte(map, node.start_byte);
        node.end_byte = map_byte(map, node.end_byte);
        convert_outline(map, &mut node.children);
    }
}

fn split_highlights_by_line(
    source: &str,
    highlights: &[HighlightToken],
) -> Vec<Vec<HighlightToken>> {
    // 先计算每行的起始和结束 UTF-16 offset
    let mut line_boundaries: Vec<(u64, u64)> = Vec::new();
    let mut line_start: u64 = 0;
    let mut pos: u64 = 0; // 独立的遍历游标
    for ch in source.chars() {
        let utf16_len = ch.len_utf16() as u64;
        pos += utf16_len;
        if ch == '\n' {
            // 行结束（包含换行符），下一行从换行符后开始
            line_boundaries.push((line_start, pos));
            line_start = pos;
        }
    }
    //最后一行（如果文件末尾没有换行符）
    if line_start < pos {
        line_boundaries.push((line_start, pos));
    } else if line_start == pos && line_start > 0 {
        line_boundaries.push((line_start, pos));
    }

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

fn build_outline(node: tree_sitter::Node, source: &[u8]) -> OutlineNode {
    let kind = node.kind().to_string();
    let name = extract_name(node, source);
    let detail = extract_detail(node, source);
    let start_byte = node.start_byte() as u64;
    let end_byte = node.end_byte() as u64;

    let mut children = Vec::new();
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            let child = cursor.node();
            if child.is_named() && child.child_count() > 0 {
                children.push(build_outline(child, source));
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }

    OutlineNode {
        kind,
        name,
        detail,
        start_byte,
        end_byte,
        children,
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

    let highlights = {
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
        let mut cursor = root.walk();
        if cursor.goto_first_child() {
            loop {
                let node = cursor.node();
                if node.is_named() && node.child_count() > 0 {
                    top.push(build_outline(node, source_bytes));
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        debug_log!("[RUST] outline count: {}", top.len());
        top
    };

    let mut result = CodeParseResult {
        highlights_by_line: Vec::new(),
        outline,
    };

    //转为UTF-16
    let mut highlights = highlights;
    if let Some(map) = build_byte_to_utf16_map(&source) {
        convert_highlights(&map, &mut highlights);
        convert_outline(&map, &mut result.outline);
    }
    // 如果 map 是 None（纯 ASCII），offsets 保持不变

    result.highlights_by_line = split_highlights_by_line(&source, &highlights);

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
        assert!(!result.highlights_by_line.is_empty(), "expected some highlights");
        // 至少应该有 keyword (using, namespace, class, static, void, string), type, function 等
        let total_tokens: usize = result.highlights_by_line.iter().map(|t| t.len()).sum();
        assert!(total_tokens > 5, "expected more than 5 tokens, got {}", total_tokens);
    }
}
