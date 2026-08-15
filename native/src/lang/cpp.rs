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
(nullptr) @constant.builtin

; KEYWORDS
[
  "auto"
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
  "export"
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
  "signed"
  "sizeof"
  "static"
  "static_assert"
  "struct"
  "switch"
  "template"
  "this"
  "throw"
  "try"
  "typedef"
  "typeid"
  "typename"
  "union"
  "unsigned"
  "using"
  "virtual"
  "void"
  "volatile"
  "while"
  "int"
  "long"
  "short"
  "char"
  "float"
  "double"
  "bool"
  "wchar_t"
  "nullptr"
  "const_cast"
  "dynamic_cast"
  "reinterpret_cast"
  "static_cast"
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
] @keyword

; PREPROCESSOR
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
(template_argument
  (type_identifier) @type)
(template_parameter
  (type_identifier) @type)

; FIELD ACCESS
(field_expression
  field: (field_identifier) @property)

; NAMESPACE
(namespace_definition
  name: (identifier) @namespace)
(namespace_definition
  name: (qualified_identifier
    (identifier) @namespace))

; ENUM CONSTANTS
(enumerator
  name: (identifier) @constant.builtin)

; PARAMETERS
(parameter_declaration
  declarator: (identifier) @variable)

; IDENTIFIER
(identifier) @identifier
"##;