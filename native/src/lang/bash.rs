use std::sync::LazyLock;
use crate::lang::GrammarDef;

pub static GRAMMAR: LazyLock<GrammarDef> = grammar!(tree_sitter_bash::LANGUAGE.into(), HIGHLIGHT_QUERY);

const HIGHLIGHT_QUERY: &str = r##"
; STRINGS & LITERALS
[
  (string)
  (raw_string)
  (heredoc_body)
  (heredoc_start)
] @string

; COMMENTS
(comment) @comment

; NUMBERS
(number) @number

; KEYWORDS
[
  "case"
  "do"
  "done"
  "elif"
  "else"
  "esac"
  "export"
  "fi"
  "for"
  "function"
  "if"
  "in"
  "select"
  "then"
  "unset"
  "until"
  "while"
] @keyword

; COMMAND NAMES
(command_name) @function

; FUNCTION DEFINITIONS
(function_definition
  name: (word) @function)

; VARIABLE EXPANSIONS
(variable_name) @variable

; VARIABLE ASSIGNMENTS
(variable_assignment
  name: (variable_name) @variable)

; SPECIAL VARIABLES
(special_variable_name) @constant.builtin

; FILE REDIRECTION
(file_redirect
  descriptor: (file_descriptor) @number)

; FILE DESCRIPTORS
(file_descriptor) @number

; OPERATORS
[
  "$"
  "&&"
  ">"
  ">>"
  "<"
  "|"
] @operator

; IDENTIFIERS (fallback)
(word) @identifier
"##;