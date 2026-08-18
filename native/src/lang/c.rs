use std::sync::LazyLock;
use crate::lang::GrammarDef;

pub static GRAMMAR: LazyLock<GrammarDef> = grammar!(tree_sitter_c::LANGUAGE.into(), HIGHLIGHT_QUERY);

const HIGHLIGHT_QUERY: &str = r##"
; COMMENTS
(comment) @comment

; STRINGS
(string_literal) @string
(escape_sequence) @escape
(system_lib_string) @string

; NUMBERS
(number_literal) @number

; CHAR
(char_literal) @string

; KEYWORDS
[
  "break"
  "case"
  "const"
  "continue"
  "default"
  "do"
  "else"
  "enum"
  "extern"
  "for"
  "goto"
  "if"
  "inline"
  "register"
  "return"
  "sizeof"
  "static"
  "struct"
  "switch"
  "typedef"
  "union"
  "volatile"
  "while"
  "#include"
  "#define"
  "#ifdef"
  "#ifndef"
  "#endif"
  "#if"
  "#else"
  "#elif"
] @keyword

; PRIMITIVE TYPES (int, void, char, etc. are primitive_type nodes, not tokens)
(primitive_type) @type
(sized_type_specifier) @type

; PREPROCESSOR (包括 #pragma/#error/#undef 等未单独列出的指令)
(preproc_directive) @keyword
(preproc_include) @keyword
(preproc_def) @keyword
(preproc_function_def
  name: (identifier) @function)

; FUNCTION DEFINITIONS
(function_definition
  declarator: (function_declarator
    declarator: (identifier) @function))

; FUNCTION CALLS
(call_expression
  function: (identifier) @function)

; TYPE IDENTIFIER
(type_identifier) @type

; FIELD ACCESS
(field_expression
  field: (field_identifier) @property)

; ENUM CONSTANTS
(enumerator
  name: (identifier) @constant.builtin)

; PARAMETERS
(parameter_declaration
  declarator: (identifier) @variable)

; IDENTIFIER
(identifier) @identifier
"##;