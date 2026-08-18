use std::sync::LazyLock;
use crate::lang::GrammarDef;

pub static GRAMMAR: LazyLock<GrammarDef> = grammar!(tree_sitter_rust::LANGUAGE.into(), HIGHLIGHT_QUERY);
const HIGHLIGHT_QUERY: &str = r##"
; COMMENTS
(line_comment) @comment
(block_comment) @comment

; STRINGS & LITERALS
(string_literal) @string
(raw_string_literal) @string
(char_literal) @string
(escape_sequence) @escape

; NUMBERS
(integer_literal) @number
(float_literal) @number

; BOOLEANS
(boolean_literal) @constant.builtin

; KEYWORDS
[
  "as"
  "async"
  "await"
  "break"
  "const"
  "continue"
  "dyn"
  "else"
  "enum"
  "extern"
  "false"
  "fn"
  "for"
  "if"
  "impl"
  "in"
  "let"
  "loop"
  "match"
  "mod"
  "move"
  "pub"
  "ref"
  "return"
  "static"
  "struct"
  "trait"
  "true"
  "type"
  "union"
  "unsafe"
  "use"
  "where"
  "while"
  "yield"
] @keyword

; OPERATORS
[
  "!"
  "!="
  "%"
  "%="
  "&"
  "&&"
  "&="
  "*"
  "*="
  "+"
  "+="
  "-"
  "-="
  "->"
  ".."
  "..="
  "..."
  "/"
  "/="
  ":"
  "::"
  ";"
  "<"
  "<<"
  "<<="
  "<="
  "="
  "=="
  "=>"
  ">"
  ">="
  ">>"
  ">>="
  "?"
  "@"
  "^"
  "^="
  "|"
  "||"
  "|="
] @operator

; FUNCTION DEFINITIONS
(function_item
  name: (identifier) @function)

; FUNCTION CALLS
(call_expression
  function: (identifier) @function)

; MACRO CALLS
(macro_invocation
  macro: (identifier) @function.builtin)

; MACRO DEFINITIONS
(macro_definition
  name: (identifier) @function.builtin)

; METHOD CALLS
(call_expression
  function: (field_expression
    field: (field_identifier) @function.method))

; METHOD DEFINITIONS
(impl_item
  body: (declaration_list
    (function_item
      name: (identifier) @function.method)))

; TRAIT IMPLEMENTATIONS
(impl_item
  trait: (type_identifier) @type)

; TYPE IDENTIFIERS
(type_identifier) @type

; TYPE ARGUMENTS (generics)
(type_arguments
  (type_identifier) @type)

; TYPE BINDINGS (associated types)
(type_binding
  name: (type_identifier) @type)

; FIELD IDENTIFIERS (struct fields)
(field_identifier) @property

; FIELD ACCESS
(field_expression
  field: (field_identifier) @property)

; STRUCT LITERAL FIELD NAMES
(struct_expression
  body: (field_initializer_list
    (field_initializer
      field: (field_identifier) @property)))

; ENUM VARIANTS
(scoped_identifier
  name: (identifier) @constant.builtin)

; NAMESPACE (module paths)
(scoped_identifier
  path: (identifier) @namespace)

; USE STATEMENTS
(use_declaration
  (scoped_identifier
    path: (identifier) @namespace))

; LIFETIME
(lifetime) @label

; ATTRIBUTES
(attribute_item) @comment
(inner_attribute_item) @comment

; FORMAT STRINGS (macro arguments)
(macro_invocation
  (token_tree
    (string_literal) @string))

; VARIABLES
(let_declaration
  pattern: (identifier) @variable)

; MUTABLE KEYWORD
(let_declaration
  (mutable_specifier) @keyword)

; FUNCTION PARAMETERS
(parameters
  (parameter
    pattern: (identifier) @variable))

; CLOSURE PARAMETERS
(closure_parameters
  (parameter
    pattern: (identifier) @variable))

; SUPER KEYWORD
(super) @keyword

; SELF PARAMETER
(self_parameter) @variable

; SELF KEYWORD IN USE PATHS
(use_list
  (self) @keyword)

(scoped_use_list
  (self) @keyword)

(scoped_identifier
  (self) @keyword)

; SELF VALUE
(self) @variable.builtin

; IDENTIFIER FALLBACK (very last)
(identifier) @identifier
"##;