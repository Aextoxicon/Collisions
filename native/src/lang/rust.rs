use std::sync::LazyLock;
use crate::lang::GrammarDef;

pub static GRAMMAR: LazyLock<GrammarDef> = grammar!(tree_sitter_rust::LANGUAGE.into(), HIGHLIGHT_QUERY, OUTLINE_QUERY);

const OUTLINE_QUERY: &str = r##"
; Outline queries for Rust
(function_item name: (identifier) @function)
(struct_item name: (type_identifier) @type)
(enum_item name: (type_identifier) @type)
(trait_item name: (type_identifier) @type)
(impl_item "impl" @keyword)
(mod_item name: (identifier) @namespace)
"##;

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
  "crate"
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
  "mut"
  "pub"
  "ref"
  "return"
  "self"
  "Self"
  "static"
  "struct"
  "super"
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
  "~"
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
(struct_literal
  (field_initializer
    (field_identifier) @property))

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

; MUTABLE VARIABLES
(let_declaration
  pattern: (mutable_specifier
    (identifier) @variable))

; MATCH PATTERNS
(match_arm
  pattern: (identifier) @variable)

; FUNCTION PARAMETERS
(parameters
  (parameter
    pattern: (identifier) @variable))

; CLOSURE PARAMETERS
(closure_parameters
  (closure_parameter
    pattern: (identifier) @variable))

; SELF PARAMETER
(self_parameter) @variable

; IDENTIFIER FALLBACK (very last)
(identifier) @identifier
"##;