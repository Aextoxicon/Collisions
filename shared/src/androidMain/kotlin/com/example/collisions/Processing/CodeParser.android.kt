package com.example.collisions.Processing

import uniffi.uniffi_code_parser.parseCode as uniffiParseCode
import uniffi.uniffi_code_parser.CodeParseResult as UniffiCodeParseResult

actual fun parseCode(source: String, extension: String): CodeParseResult {
    val language = mapLanguage(extension)
    if (language == "text") {
        return CodeParseResult.PlainText(
            language = language,
            content = source,
        )
    }
    return try {
        val result: UniffiCodeParseResult = uniffiParseCode(source, extension)
        CodeParseResult.Code(
            language = language,
            content = source,
            highlightsByLine = result.highlightsByLine.map { line ->
                line.map { token ->
                    HighlightToken(
                        startByte = token.startByte.toLong(),
                        endByte = token.endByte.toLong(),
                        kind = HighlightToken.mapKind(token.kind.name),
                    )
                }
            },
            outline = result.outline.map { it.toKt() },
        )
    } catch (e: UnsatisfiedLinkError) {
        // 原生库不可用时，返回空解析结果
        CodeParseResult.Code(
            language = language,
            content = source,
            highlightsByLine = emptyList(),
            outline = emptyList(),
        )
    }
}

private fun uniffi.uniffi_code_parser.OutlineNode.toKt(): com.example.collisions.Processing.OutlineNode =
    com.example.collisions.Processing.OutlineNode(
        kind = kind,
        name = name,
        detail = detail,
        startByte = startByte.toLong(),
        endByte = endByte.toLong(),
        children = children.map { it.toKt() },
    )

private fun mapLanguage(extension: String): String = when (extension.lowercase().trimStart('.')) {
    "kt", "kts" -> "kotlin"
    "java" -> "java"
    "swift" -> "swift"
    "c" -> "c"
    "cpp", "cc", "cxx" -> "cpp"
    "h", "hpp" -> "cpp"
    "rs" -> "rust"
    "py" -> "python"
    "js", "jsx" -> "javascript"
    "ts", "tsx" -> "typescript"
    "html", "htm" -> "html"
    "css" -> "css"
    "json" -> "json"
    "xml" -> "xml"
    "yaml", "yml" -> "yaml"
    "toml" -> "toml"
    "sh", "bash" -> "bash"
    "go" -> "go"
    "md", "markdown" -> "markdown"
    "txt" -> "text"
    else -> "text"
}