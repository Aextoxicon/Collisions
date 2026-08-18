use std::sync::LazyLock;
use crate::lang::GrammarDef;

pub static GRAMMAR: LazyLock<GrammarDef> = grammar!(tree_sitter_cpp::LANGUAGE.into(), HIGHLIGHT_QUERY);

const HIGHLIGHT_QUERY: &str = r##"
; COMMENTS
(comment) @comment

; STRINGS
(string_literal) @string
(raw_string_literal) @string
(escape_sequence) @escape

; NUMBERS
(number_literal) @number

; BOOLEANS
(true) @constant.builtin
(false) @constant.builtin
(null "nullptr" @constant.builtin)

; KEYWORDS
[
  "break"
  "case"
  "catch"
  "class"
  "const"
  "constexpr"
  "continue"
  "decltype"
  "default"
  "delete"
  "do"
  "else"
  "enum"
  "explicit"
  "extern"
  "for"
  "friend"
  "goto"
  "if"
  "inline"
  "mutable"
  "namespace"
  "new"
  "noexcept"
  "operator"
  "override"
  "private"
  "protected"
  "public"
  "register"
  "return"
  "sizeof"
  "static"
  "static_assert"
  "struct"
  "switch"
  "template"
  "throw"
  "try"
  "typedef"
  "typename"
  "union"
  "using"
  "virtual"
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

; PRIMITIVE TYPES
(primitive_type) @type
(sized_type_specifier) @type

; PREPROCESSOR (包括 #pragma/#error/#undef 等)
(preproc_directive) @keyword
(preproc_include) @keyword
(preproc_def) @keyword

; FUNCTION DEFINITIONS
(function_definition
  declarator: (function_declarator
    declarator: (identifier) @function))

; FUNCTION CALLS
(call_expression
  function: (identifier) @function)

; METHOD CALLS
(call_expression
  function: (field_expression
    field: (field_identifier) @function.method))

; TYPE IDENTIFIER
(type_identifier) @type

; TEMPLATE
(template_instantiation
  (type_identifier) @type)

; FIELD ACCESS
(field_expression
  field: (field_identifier) @property)

; NAMESPACE
(namespace_definition
  name: (namespace_identifier) @namespace)

; ENUM CONSTANTS
(enumerator
  name: (identifier) @constant.builtin)

; PARAMETERS
(parameter_declaration
  declarator: (identifier) @variable)

; IDENTIFIER
(identifier) @identifier
"##;