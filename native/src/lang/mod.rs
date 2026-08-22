use std::sync::LazyLock;

// 消除每个语言文件中的模板代码（LazyLock + GrammarDef 构造）
// 用法：
//   grammar!(tree_sitter_go::LANGUAGE, HIGHLIGHT_QUERY)
macro_rules! grammar {
    ($lang:expr, $highlight:ident) => {
        ::std::sync::LazyLock::new(|| {
            let query = ::tree_sitter::Query::new(&$lang, $highlight)
                .expect("invalid highlight query");
            $crate::lang::GrammarDef {
                language: $lang,
                compiled_query: query,
            }
        })
    };
}

mod c;
mod cpp;
mod go;
mod python;
mod javascript;
mod typescript;
mod bash;
mod csharp;
mod java;
mod json;
mod css;
mod rust;

pub struct GrammarDef {
    pub language: tree_sitter::Language,
    /// 预编译的 Query
    pub compiled_query: tree_sitter::Query,
}

// 按文件扩展名查找对应的 grammar 定义
pub fn get_grammar(ext: &str) -> Option<&'static LazyLock<GrammarDef>> {
    match ext {
        ".c" => Some(&c::GRAMMAR),
        ".h" => Some(&c::GRAMMAR),
        ".cpp" | ".cc" | ".cxx" => Some(&cpp::GRAMMAR),
        ".hpp" => Some(&cpp::GRAMMAR),
        ".go" => Some(&go::GRAMMAR),
        ".py" => Some(&python::GRAMMAR),
        ".js" | ".mjs" | ".cjs" => Some(&javascript::GRAMMAR),
        ".ts" => Some(&typescript::GRAMMAR_TS),
        ".tsx" => Some(&typescript::GRAMMAR_TSX),
        ".sh" | ".bash" | ".zsh" => Some(&bash::GRAMMAR),
        ".cs" => Some(&csharp::GRAMMAR),
        ".java" => Some(&java::GRAMMAR),
        ".json" => Some(&json::GRAMMAR),
        ".css" => Some(&css::GRAMMAR),
        ".rs" => Some(&rust::GRAMMAR),
        _ => None,
    }
}
