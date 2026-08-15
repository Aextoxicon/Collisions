use std::sync::LazyLock;
use crate::lang::GrammarDef;

pub static GRAMMAR: LazyLock<GrammarDef> = grammar!(tree_sitter_json::LANGUAGE.into(), HIGHLIGHT_QUERY);

const HIGHLIGHT_QUERY: &str = r##"
; JSON highlights

; Strings
(string) @string
(escape_sequence) @escape

; Numbers
(number) @number

; Booleans & null
(true) @constant.builtin
(false) @constant.builtin
(null) @constant.builtin

; Keys (property names)
(pair key: (string) @property)

; Comments (JSONC / JSON with comments)
(comment) @comment
"##;