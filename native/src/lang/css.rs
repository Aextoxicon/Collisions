use std::sync::LazyLock;
use crate::lang::GrammarDef;

pub static GRAMMAR: LazyLock<GrammarDef> = grammar!(tree_sitter_css::LANGUAGE.into(), HIGHLIGHT_QUERY);

const HIGHLIGHT_QUERY: &str = r##"
; CSS highlights

; Selectors
(selector) @constant

; Properties
(property_name) @property

; Values
(property_value) @string

; Numbers
(integer_value) @number
(float_value) @number

; Keywords
[
  "important"
  "inherit"
  "initial"
  "unset"
] @keyword

; Colors
(color_value) @constant.builtin

; Pseudo classes/elements
(pseudo_class_selector) @function.builtin
(pseudo_element_selector) @function.builtin

; ID selectors
(id_selector) @constant.builtin

; Class selectors
(class_selector) @constant.builtin

; Comment
(comment) @comment

; Tag name selector
(tag_name) @type

; Attribute selectors
(attribute_selector) @attribute

; At-rules
(at_rule) @keyword
"##;