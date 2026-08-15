use std::sync::LazyLock;
use crate::lang::GrammarDef;

pub static GRAMMAR: LazyLock<GrammarDef> = grammar!(tree_sitter_java::LANGUAGE.into(), HIGHLIGHT_QUERY);

const HIGHLIGHT_QUERY: &str = r##"
; COMMENTS
(line_comment) @comment
(block_comment) @comment

; STRINGS
(string_literal) @string
(escape_sequence) @escape
(char_literal) @string

; NUMBERS
(integer_literal) @number
(floating_point_literal) @number

; BOOLEANS
(true) @constant.builtin
(false) @constant.builtin
(null_literal) @constant.builtin

; KEYWORDS
[
  "abstract"
  "assert"
  "break"
  "case"
  "catch"
  "class"
  "continue"
  "default"
  "do"
  "else"
  "enum"
  "extends"
  "final"
  "finally"
  "for"
  "if"
  "implements"
  "import"
  "instanceof"
  "interface"
  "native"
  "new"
  "package"
  "private"
  "protected"
  "public"
  "return"
  "static"
  "strictfp"
  "super"
  "switch"
  "synchronized"
  "this"
  "throw"
  "throws"
  "transient"
  "try"
  "var"
  "void"
  "volatile"
  "while"
  "module"
  "requires"
  "exports"
  "opens"
  "uses"
  "provides"
  "to"
  "with"
  "transitive"
  "record"
  "yield"
  "sealed"
  "permits"
  "non-sealed"
] @keyword

; PRIMITIVE TYPES
[
  "boolean"
  "byte"
  "char"
  "double"
  "float"
  "int"
  "long"
  "short"
] @type.builtin

; ANNOTATIONS
(annotation
  (identifier) @attribute)
(marker_annotation
  (identifier) @attribute)

; CLASS DECLARATIONS
(class_declaration
  name: (identifier) @type)

; INTERFACE DECLARATIONS
(interface_declaration
  name: (identifier) @type)

; ENUM DECLARATIONS
(enum_declaration
  name: (identifier) @type)

; METHOD DECLARATIONS
(method_declaration
  name: (identifier) @function)

; METHOD CALLS
(method_invocation
  name: (identifier) @function)

; OBJECT CREATION
(object_creation_expression
  type: (type_identifier) @type)

; TYPE IDENTIFIER
(type_identifier) @type

; FIELD ACCESS
(field_access
  field: (identifier) @property)

; CONSTANTS
(constant_declaration
  name: (identifier) @constant)

; PARAMETERS
(formal_parameter
  name: (identifier) @variable)

; LOCAL VARIABLES
(variable_declarator
  name: (identifier) @variable)

; IDENTIFIER
(identifier) @identifier
"##;