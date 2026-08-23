use uniffi_code_parser::{
    parse_code,
    CodeParseResult,
    HighlightToken,
    OutlineNode,
};

#[test]
fn test_parse_go_code() {
    let source = r#"package main

import "fmt"

func main() {
    fmt.Println("hello world")
}
"#;
    let result: CodeParseResult = parse_code(source.to_string(), ".go".to_string());
    eprintln!("=== HIGHLIGHTS BY LINE ===");
    for (i, line_tokens) in result.highlights_by_line.iter().enumerate() {
        let tokens: &Vec<HighlightToken> = line_tokens;
        eprintln!("  line {}: {} tokens", i, tokens.len());
        for h in tokens {
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
        let result: CodeParseResult = parse_code(source.to_string(), ext.to_string());
        eprintln!("=== {} ===", ext);
        eprintln!("  lines: {} (source has {} lines)", result.highlights_by_line.len(), source.lines().count());
        let total_tokens: usize = result.highlights_by_line.iter().map(|t: &Vec<HighlightToken>| t.len()).sum();
        eprintln!("  total tokens: {}", total_tokens);

        // 简单断言：至少有一个 token（C 语言至少能识别 #include 或 comment）
        // 注意：JSON 标准语法里 // 不是注释，只要没 panic 就算通过
        assert!(
            total_tokens > 0,
            "{}: expected at least 1 highlight token, got 0",
            ext
        );

        // 打印前几行 token 种类用于调试
        for (i, line_tokens) in result.highlights_by_line.iter().enumerate().take(3) {
            let line: &Vec<HighlightToken> = line_tokens;
            let kinds: Vec<String> = line.iter().map(|t: &HighlightToken| format!("{:?}", t.kind)).collect();
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
    let result: CodeParseResult = parse_code(source.to_string(), ".py".to_string());
    eprintln!("=== PYTHON HIGHLIGHTS BY LINE ===");
    for (i, line_tokens) in result.highlights_by_line.iter().enumerate() {
        let tokens: &Vec<HighlightToken> = line_tokens;
        eprintln!("  line {}: {} tokens", i, tokens.len());
        for h in tokens {
            eprintln!("    [{:?}] offset {}-{}", h.kind, h.start_byte, h.end_byte);
        }
    }
    assert!(!result.highlights_by_line.is_empty(), "expected some highlights");
    // 至少应该有 keyword (import, def), string, identifier 等
    let total_tokens: usize = result.highlights_by_line.iter().map(|t: &Vec<HighlightToken>| t.len()).sum();
    assert!(total_tokens > 5, "expected more than 5 tokens, got {}", total_tokens);
}

#[test]
fn test_all_queries_compile() {
    // 验证每个 grammar 的 highlight query 都能成功编译
    let extensions = [".c", ".h", ".cpp", ".hpp", ".go", ".py", ".js", ".mjs", ".cjs", ".ts", ".tsx", ".sh", ".bash", ".zsh", ".cs", ".java", ".json", ".css", ".rs"];
    let mut failures = Vec::new();
    for ext in extensions {
        let grammar = uniffi_code_parser::lang::get_grammar(ext)
            .unwrap_or_else(|| panic!("no grammar for {}", ext));
        let names = grammar.compiled_query.capture_names();
        eprintln!("[QUERY OK] {} ({} captures)", ext, names.len());
        if names.is_empty() {
            failures.push(format!("{}: no captures", ext));
        }
    }
    assert!(failures.is_empty(), "query failures:\n{}", failures.join("\n"));
}

/// 在 outline 树中递归查找指定 kind 的节点
fn outline_contains(outline: &[OutlineNode], kind: &str) -> bool {
    for node in outline {
        if node.kind == kind {
            return true;
        }
        if outline_contains(&node.children, kind) {
            return true;
        }
    }
    false
}

#[test]
fn test_unicode_source_utf16_offsets() {
    // 含中文/emoji 的源码，验证 UTF-16 偏移映射不破坏高亮
    let source = r#"import os

def hello():
    name = "你好世界👋"
    print(name)  # 打印名字
    return name
"#;
    let result: CodeParseResult = parse_code(source.to_string(), ".py".to_string());
    eprintln!("=== UNICODE HIGHLIGHTS BY LINE ===");
    for (i, line_tokens) in result.highlights_by_line.iter().enumerate() {
        let tokens: &Vec<HighlightToken> = line_tokens;
        eprintln!("  line {}: {} tokens", i, tokens.len());
        for h in tokens {
            eprintln!("    [{:?}] offset {}-{}", h.kind, h.start_byte, h.end_byte);
        }
    }
    // 至少有 6 行有高亮（import, def, string, comment, keyword, return）
    assert!(!result.highlights_by_line.is_empty(), "expected some highlights");
    let total_tokens: usize = result.highlights_by_line.iter().map(|t: &Vec<HighlightToken>| t.len()).sum();
    assert!(total_tokens > 5, "expected more than 5 tokens with unicode, got {}", total_tokens);
    let has_string = result.highlights_by_line.iter().flatten().any(|t| matches!(t.kind, uniffi_code_parser::HighlightTokenKind::StringLiteral));
    assert!(has_string, "expected at least one StringLiteral highlight with unicode source");
}

#[test]
fn test_unicode_source_rust() {
    let source = r#"// 这是一个中文注释
fn main() {
    let x = "测试 emoji 🦀";
    println!("{}", x);
}
"#;
    let result: CodeParseResult = parse_code(source.to_string(), ".rs".to_string());
    eprintln!("=== RUST UNICODE HIGHLIGHTS BY LINE ===");
    for (i, line_tokens) in result.highlights_by_line.iter().enumerate() {
        let tokens: &Vec<HighlightToken> = line_tokens;
        eprintln!("  line {}: {} tokens", i, tokens.len());
        for h in tokens {
            eprintln!("    [{:?}] offset {}-{}", h.kind, h.start_byte, h.end_byte);
        }
    }
    assert!(!result.highlights_by_line.is_empty(), "expected some highlights");
    let has_comment = result.highlights_by_line.iter().flatten().any(|t| matches!(t.kind, uniffi_code_parser::HighlightTokenKind::Comment));
    assert!(has_comment, "expected at least one Comment highlight for Chinese comment");
    let has_string = result.highlights_by_line.iter().flatten().any(|t| matches!(t.kind, uniffi_code_parser::HighlightTokenKind::StringLiteral));
    assert!(has_string, "expected at least one StringLiteral highlight for emoji string");
}

#[test]
fn test_outline_go() {
    let source = r#"package main

func main() {
    println("hello")
}
"#;
    let result: CodeParseResult = parse_code(source.to_string(), ".go".to_string());
    eprintln!("=== GO OUTLINE ===");
    fn print_outline(nodes: &[OutlineNode], depth: usize) {
        let indent = "  ".repeat(depth);
        for node in nodes {
            eprintln!("{}{} name={:?} bytes {}-{} children={}", indent, node.kind, node.name, node.start_byte, node.end_byte, node.children.len());
            print_outline(&node.children, depth + 1);
        }
    }
    print_outline(&result.outline, 0);
    assert!(outline_contains(&result.outline, "package_clause"), "expected package_clause in Go outline");
    assert!(outline_contains(&result.outline, "function_declaration"), "expected function_declaration in Go outline");
    // 验证 function_definition 的 name 是 "main"
    let has_main = result.outline.iter().any(|n| n.name == "main")
        || result.outline.iter().any(|n| n.children.iter().any(|c| c.name == "main"));
    assert!(has_main, "expected function named 'main' in Go outline");
}

#[test]
fn test_outline_python() {
    let source = r#"class Greeter:
    def greet(self):
        print("hello")
"#;
    let result: CodeParseResult = parse_code(source.to_string(), ".py".to_string());
    eprintln!("=== PYTHON OUTLINE ===");
    for node in &result.outline {
        eprintln!("{} name={:?} children={}", node.kind, node.name, node.children.len());
        for c in &node.children {
            eprintln!("  {} name={:?}", c.kind, c.name);
        }
    }
    assert!(outline_contains(&result.outline, "class_definition"), "expected class_definition in Python outline");
    assert!(outline_contains(&result.outline, "function_definition"), "expected function_definition in Python outline");
    let has_greeter = result.outline.iter().any(|n| n.name == "Greeter");
    assert!(has_greeter, "expected class named 'Greeter' in Python outline");
}

#[test]
fn test_outline_java() {
    let source = r#"public class Main {
    public static void main(String[] args) {
    }
}
"#;
    let result: CodeParseResult = parse_code(source.to_string(), ".java".to_string());
    eprintln!("=== JAVA OUTLINE ===");
    for node in &result.outline {
        eprintln!("{} name={:?} children={}", node.kind, node.name, node.children.len());
        for c in &node.children {
            eprintln!("  {} name={:?}", c.kind, c.name);
        }
    }
    assert!(outline_contains(&result.outline, "class_declaration"), "expected class_declaration in Java outline");
    // Java 中 main 方法可能是 method_declaration 或 function_declaration
    let has_main = outline_contains(&result.outline, "method_declaration") || outline_contains(&result.outline, "function_declaration");
    assert!(has_main, "expected method/function declaration in Java outline");
}

#[test]
fn test_outline_rust() {
    let source = r#"use std::fmt;

mod utils {
    pub fn helper() -> i32 {
        42
    }
}

fn main() {
    let x = 1;
}
"#;
    let result: CodeParseResult = parse_code(source.to_string(), ".rs".to_string());
    eprintln!("=== RUST OUTLINE ===");
    fn print_outline(nodes: &[OutlineNode], depth: usize) {
        let indent = "  ".repeat(depth);
        for node in nodes {
            eprintln!("{}{} name={:?} children={}", indent, node.kind, node.name, node.children.len());
            print_outline(&node.children, depth + 1);
        }
    }
    print_outline(&result.outline, 0);
    assert!(outline_contains(&result.outline, "function_item"), "expected function_item in Rust outline");
    // Rust 的 mod 项可能是 mod_item
    let has_mod = outline_contains(&result.outline, "mod_item");
    assert!(has_mod, "expected mod_item in Rust outline");
}
