package com.example.collisions.Processing

object FileProcessor {
    fun process(content: String, extension: String): CodeParseResult {
        val language = mapLanguage(extension)
        return when (language) {
            "text", "markdown" -> {
                CodeParseResult.PlainText(language = language, content = content)
            }
            else -> {
                parseCode(content, extension)
            }
        }
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
