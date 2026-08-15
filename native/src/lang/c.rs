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
  "auto"
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
  "restrict"
  "return"
  "signed"
  "sizeof"
  "static"
  "struct"
  "switch"
  "typedef"
  "union"
  "unsigned"
  "void"
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
  "#pragma"
  "#error"
  "#undef"
  "#line"
  "int"
  "long"
  "short"
  "char"
  "float"
  "double"
] @keyword

; PREPROCESSOR
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