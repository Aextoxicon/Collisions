use std::sync::LazyLock;
use crate::lang::GrammarDef;

pub static GRAMMAR: LazyLock<GrammarDef> = grammar!(tree_sitter_c_sharp::LANGUAGE.into(), HIGHLIGHT_QUERY);

const HIGHLIGHT_QUERY: &str = r##"
;COMMENTS

(comment) @comment

;STRINGS & LITERALS

[
  (string_literal)
  (verbatim_string_literal)
  (character_literal)
] @string

;NUMBERS

[
  (integer_literal)
  (real_literal)
] @number

;PREDEFINED TYPES (bool, int, string, etc.)

(predefined_type) @type

;KEYWORDS

[
  "abstract"
  "as"
  "async"
  "await"
  "base"
  "break"
  "case"
  "catch"
  "checked"
  "class"
  "const"
  "continue"
  "default"
  "delegate"
  "do"
  "else"
  "enum"
  "event"
  "explicit"
  "extern"
  "finally"
  "fixed"
  "for"
  "foreach"
  "goto"
  "if"
  "implicit"
  "in"
  "interface"
  "internal"
  "is"
  "lock"
  "namespace"
  "new"
  "operator"
  "out"
  "override"
  "params"
  "private"
  "protected"
  "public"
  "readonly"
  "ref"
  "return"
  "sealed"
  "sizeof"
  "stackalloc"
  "static"
  "struct"
  "switch"
  "this"
  "throw"
  "try"
  "typeof"
  "unchecked"
  "unsafe"
  "using"
  "virtual"
  "volatile"
  "while"
] @keyword

;FUNCTION CALLS

(invocation_expression
  function: (identifier) @function)

(invocation_expression
  function: (member_access_expression
    name: (identifier) @function.method))

;METHOD DEFINITIONS

(method_declaration
  name: (identifier) @function)

;CLASS/STRUCT DEFINITIONS

(class_declaration
  name: (identifier) @type)

(struct_declaration
  name: (identifier) @type)

(interface_declaration
  name: (identifier) @type)

(enum_declaration
  name: (identifier) @type)

(record_declaration
  name: (identifier) @type)

;NAMESPACE

(namespace_declaration
  name: (identifier) @namespace)

;ATTRIBUTES

(attribute
  name: (identifier) @function)

;PROPERTY ACCESS

(member_access_expression
  name: (identifier) @property)

;VARIABLE DECLARATIONS

(variable_declaration
  (variable_declarator
    (identifier) @variable))

;PARAMETERS

(parameter
  name: (identifier) @variable)

;OPERATORS

[
  "+"
  "-"
  "*"
  "/"
  "%"
  "="
  "+="
  "-="
  "*="
  "/="
  "%="
  "=="
  "!="
  "<"
  ">"
  "<="
  ">="
  "&&"
  "||"
  "!"
  "&"
  "|"
  "^"
  "~"
  "<<"
  ">>"
  "??"
  "=>"
  "::"
  "++"
  "--"
  "??="
] @operator

;PUNCTUATION

[
  ";"
  "."
  ","
  "("
  ")"
  "["
  "]"
  "{"
  "}"
] @punctuation

;IDENTIFIERS (fallback)

(identifier) @identifier
"##;