use std::sync::LazyLock;
use crate::lang::GrammarDef;

pub static GRAMMAR: LazyLock<GrammarDef> = grammar!(tree_sitter_swift::LANGUAGE.into(), tree_sitter_swift::HIGHLIGHTS_QUERY);