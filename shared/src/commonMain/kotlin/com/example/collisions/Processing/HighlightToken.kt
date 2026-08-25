package com.example.collisions.Processing

// 是相对于所在行的UTF-16偏移
data class HighlightToken(
    val startByte: Long,
    val endByte: Long,
    val kind: String,
) {
    companion object {
        // uniffi 枚举的 name 是 SCREAMING_SNAKE_CASE（如 STRING_LITERAL）
        // 映射到可读的 tree-sitter 风格高亮类别
        fun mapKind(kind: String): String = when (kind) {
            "KEYWORD" -> "keyword"
            "STRING_LITERAL" -> "string"
            "COMMENT" -> "comment"
            "FUNCTION" -> "function"
            "FUNCTION_BUILTIN" -> "function.builtin"
            "FUNCTION_METHOD" -> "function.method"
            "TYPE" -> "type"
            "NUMBER" -> "number"
            "OPERATOR" -> "operator"
            "IDENTIFIER" -> "identifier"
            "VARIABLE" -> "variable"
            "PROPERTY" -> "property"
            "PUNCTUATION" -> "punctuation"
            "ESCAPE" -> "escape"
            "CONSTANT_BUILTIN" -> "constant.builtin"
            "LABEL" -> "label"
            "NAMESPACE" -> "namespace"
            // 新增
            "BOOLEAN" -> "boolean"
            "ATTRIBUTE" -> "attribute"
            "CONDITIONAL" -> "conditional"
            "REPEAT" -> "repeat"
            "INCLUDE" -> "include"
            "KEYWORD_FUNCTION" -> "keyword.function"
            "EXCEPTION" -> "exception"
            "UNKNOWN" -> "identifier"    // UNKNOWN = "identifier"
            else -> kind.lowercase() // else 直接小写
        }
    }
}