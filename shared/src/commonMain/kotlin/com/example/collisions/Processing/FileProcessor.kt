package com.example.collisions.Processing

object FileProcessor {
    private val rustSupportedExtensions = setOf(
        "go", "py", "js", "mjs", "cjs", "ts", "tsx", "sh", "bash", "zsh", "cs",
    )

    fun process(content: String, extension: String): CodeParseResult {
        val language = mapLanguage(extension)
        return when (language) {
            "text", "markdown" -> {
                CodeParseResult.PlainText(language = language, content = content)
            }
            else -> {
                val rustExt = normalizeForRust(extension)
                if (rustExt == null) {
                    // 没有对应grammar，降级为纯文本渲染
                    CodeParseResult.PlainText(language = language, content = content)
                } else {
                    parseCode(content, rustExt)
                }
            }
        }
    }

    private fun normalizeForRust(extension: String): String? {
        val clean = extension.lowercase().trimStart('.')
        return if (clean in rustSupportedExtensions) ".$clean" else null
    }

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
}
