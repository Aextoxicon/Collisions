package com.example.collisions.Processing

object FileProcessor {
    private val textExtensions = setOf("txt", "md", "markdown")

    fun process(content: String, extension: String): CodeParseResult {
        val cleanExt = extension.lowercase().trimStart('.')
        if (cleanExt in textExtensions) {
            return CodeParseResult.PlainText(language = cleanExt, content = content)
        }
        // 非纯文本直接传给Rust解析，扩展名路由改成Rust侧维护
        return parseCode(content, ".$cleanExt")
    }
}