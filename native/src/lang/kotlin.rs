use std::sync::LazyLock;
use crate::lang::GrammarDef;

pub static GRAMMAR: LazyLock<GrammarDef> = grammar!(tree_sitter_kotlin_sg::LANGUAGE.into(), tree_sitter_kotlin_sg::HIGHLIGHTS_QUERY);