package com.example.collisions.Utils

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.withStyle
import com.example.collisions.Processing.CodeParseResult
import com.example.collisions.Processing.HighlightToken
import com.example.collisions.Processing.OutlineNode

object HighlightColor {
    val keyword = Color(0xFFD73A49)
    val string = Color(0xFF09622A)
    val comment = Color(0xFF6A737D)
    val function = Color(0xFF8250DF)
    val functionBuiltin = Color(0xFF005CC5)
    val functionMethod = Color(0xFF6F42C1)
    val type = Color(0xFF6F42C1)
    val number = Color(0xFF0550AE)
    val operator = Color(0xFFD73A49)
    val identifier = Color(0xFF24292E)
    val variable = Color(0xFFE36209)
    val property = Color(0xFFEC09BE)
    val punctuation = Color(0xFF24292E)
    val escape = Color(0xFFE36209)
    val constantBuiltin = Color(0xFF953800)
    val label = Color(0xFFE36209)
    val namespace = Color(0xFF28A745)
    val builtin = Color(0xFF6F42C1)
    val plainText = Color(0xFF24292E)

    private val colorMap: Map<String, Color> = mapOf(
        "keyword" to keyword,
        "string" to string,
        "comment" to comment,
        "function" to function,
        "function.builtin" to functionBuiltin,
        "function.method" to functionMethod,
        "type" to type,
        "number" to number,
        "operator" to operator,
        "builtin" to builtin,
        "identifier" to identifier,
        "variable" to variable,
        "property" to property,
        "punctuation" to punctuation,
        "escape" to escape,
        "constant.builtin" to constantBuiltin,
        "label" to label,
        "namespace" to namespace,
        // 新增语言的高亮类别
        "boolean" to keyword,
        "attribute" to keyword,
        "conditional" to keyword,
        "repeat" to keyword,
        "include" to keyword,
        "keyword.function" to functionBuiltin,
        "exception" to keyword,
    )

    fun colorFor(kind: String): Color = colorMap[kind] ?: plainText

    fun toAnnotatedString(
        parseResult: CodeParseResult,
        colorDefault: Color = plainText,
    ): AnnotatedString {
        return when (parseResult) {
            is CodeParseResult.PlainText -> {
                AnnotatedString(
                    parseResult.content,
                    spanStyle = SpanStyle(color = colorDefault),
                )
            }
            is CodeParseResult.Code -> {
                buildAnnotatedString(parseResult.content, parseResult.highlightsByLine, colorDefault)
            }
        }
    }

    private fun buildAnnotatedString(
        content: String,
        highlightsByLine: List<List<HighlightToken>>,
        colorDefault: Color,
    ): AnnotatedString {
        val builder = buildAnnotatedString {
            val lines = content.split("\n")
            for ((lineIndex, line) in lines.withIndex()) {
                val tokens = if (lineIndex < highlightsByLine.size) highlightsByLine[lineIndex] else emptyList()
                if (tokens.isEmpty()) {
                    // 该行无高亮，整行使用默认颜色
                    append(line)
                } else {
                    var pos = 0
                    for (token in tokens) {
                        val start = token.startByte.toInt()
                        val end = token.endByte.toInt()
                        // 添加 token 之前的文本（默认颜色）
                        if (start > pos) {
                            withStyle(SpanStyle(color = colorDefault)) {
                                append(line.substring(pos, start.coerceAtMost(line.length)))
                            }
                        }
                        // 添加 token 本身（高亮颜色）
                        if (start < end && start < line.length) {
                            val color = colorFor(HighlightToken.mapKind(token.kind))
                            val tokenEnd = end.coerceAtMost(line.length)
                            withStyle(SpanStyle(color = color)) {
                                append(line.substring(start, tokenEnd))
                            }
                        }
                        pos = end.coerceAtMost(line.length)
                    }
                    // 行尾剩余文本
                    if (pos < line.length) {
                        withStyle(SpanStyle(color = colorDefault)) {
                            append(line.substring(pos))
                        }
                    }
                }
                // 添加换行符（最后一行不加）
                if (lineIndex < lines.size - 1) {
                    append("\n")
                }
            }
        }
        return builder
    }
}