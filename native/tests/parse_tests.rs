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