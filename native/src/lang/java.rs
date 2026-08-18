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
(character_literal) @string

; NUMBERS
(hex_integer_literal) @number
(decimal_integer_literal) @number
(octal_integer_literal) @number
(decimal_floating_point_literal) @number
(hex_floating_point_literal) @number

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
  "switch"
  "synchronized"
  "throw"
  "throws"
  "transient"
  "try"
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
(boolean_type) @type.builtin
(integral_type) @type.builtin
(floating_point_type) @type.builtin

(void_type) @keyword

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
  declarator: (variable_declarator
    name: (identifier) @constant))

; PARAMETERS
(formal_parameter
  name: (identifier) @variable)

; LOCAL VARIABLES
(variable_declarator
  name: (identifier) @variable)

; IDENTIFIER
(identifier) @identifier
"##;