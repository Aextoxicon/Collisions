use std::sync::LazyLock;
use crate::lang::GrammarDef;

pub static GRAMMAR: LazyLock<GrammarDef> = grammar!(tree_sitter_python::LANGUAGE.into(), HIGHLIGHT_QUERY);

const HIGHLIGHT_QUERY: &str = r##"
;COMMENTS

(comment) @comment

;STRINGS & LITERALS

(string) @string

(escape_sequence) @escape

;NUMBERS

[
  (integer)
  (float)
] @number

;CONSTANTS

[
  (true)
  (false)
  (none)
] @constant.builtin

;KEYWORDS

[
  "and"
  "as"
  "assert"
  "async"
  "await"
  "break"
  "class"
  "continue"
  "def"
  "del"
  "elif"
  "else"
  "except"
  "finally"
  "for"
  "from"
  "global"
  "if"
  "import"
  "in"
  "is"
  "lambda"
  "nonlocal"
  "not"
  "or"
  "pass"
  "raise"
  "return"
  "try"
  "while"
  "with"
  "yield"
] @keyword

;OPERATORS

[
  "-"
  "-="
  "!="
  "*"
  "**"
  "**="
  "*="
  "/"
  "//"
  "//="
  "/="
  "&"
  "&="
  "%"
  "%="
  "^"
  "^="
  "+"
  "+="
  "<"
  "<<"
  "<<="
  "<="
  "<>"
  "="
  ":="
  "=="
  ">"
  ">="
  ">>"
  ">>="
  "|"
  "|="
  "|"
  "~"
  "@"
] @operator

;FUNCTION CALLS

(call
  function: (identifier) @function)

(call
  function: (attribute
    attribute: (identifier) @function.method))

;FUNCTION DEFINITIONS

(function_definition
  name: (identifier) @function)

;CLASS DEFINITIONS

(class_definition
  name: (identifier) @type)

;DECORATORS

(decorator
  "@" @operator
  (identifier) @function)

;IMPORTS

(import_statement
  name: (dotted_name
    (identifier) @namespace))

(import_from_statement
  module_name: (dotted_name
    (identifier) @namespace))

(import_from_statement
  name: (dotted_name
    (identifier) @function))

;PARAMETERS

(parameters
  (identifier) @variable)

(default_parameter
  name: (identifier) @variable)

(typed_parameter
  (identifier) @variable)

(lambda_parameters
  (identifier) @variable)

;ASSIGNMENTS

(assignment
  left: (identifier) @variable)

(augmented_assignment
  left: (identifier) @variable)

;TYPE ANNOTATIONS

(type (identifier) @type)

;ATTRIBUTES

(attribute
  attribute: (identifier) @property)

;IDENTIFIERS (fallback)

(identifier) @identifier
"##;