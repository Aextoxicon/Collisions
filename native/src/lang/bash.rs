use std::sync::LazyLock;
use crate::lang::GrammarDef;

pub static GRAMMAR: LazyLock<GrammarDef> = grammar!(tree_sitter_bash::LANGUAGE.into(), HIGHLIGHT_QUERY);

const HIGHLIGHT_QUERY: &str = r##"
;COMMENTS

(comment) @comment

;STRINGS & LITERALS

[
  (string)
  (raw_string)
] @string

(escape_sequence) @escape

;HEREDOC

(heredoc_body) @string
(heredoc_start) @operator

;NUMBERS

(number) @number

;KEYWORDS

[
  "if"
  "then"
  "elif"
  "else"
  "fi"
  "case"
  "esac"
  "for"
  "while"
  "until"
  "do"
  "done"
  "in"
  "select"
  "function"
  "time"
  "declare"
  "local"
  "export"
  "readonly"
  "unset"
  "typeset"
  "return"
  "exit"
  "break"
  "continue"
  "eval"
  "exec"
  "let"
  "shift"
  "source"
  "."
  "cd"
  "echo"
  "printf"
  "read"
  "set"
  "test"
  "["
  "]]"
  "!";
] @keyword

;COMMAND NAMES

(command_name
  (command_identifier) @function)

;VARIABLE EXPANSIONS

(variable_name) @variable

;VARIABLE ASSIGNMENTS

(variable_assignment
  name: (variable_name) @variable)

;SPECIAL VARIABLES

(special_variable_name) @constant.builtin

;TEST OPERATORS

(binary_expression
  operator: (operator) @operator)

(unary_expression
  operator: (operator) @operator)

;FILE REDIRECTION

(file_redirect
  (file_descriptor) @number
  (destination) @string)

;FUNCTION DEFINITIONS

(function_definition
  name: (word) @function)

;IDENTIFIERS (fallback)

(word) @identifier
"##;