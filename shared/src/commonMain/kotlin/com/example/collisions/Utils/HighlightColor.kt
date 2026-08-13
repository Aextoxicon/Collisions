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
    // 颜色定义（基于 One Dark Pro 配色）
    val keyword = Color(0xFFC678DD)        // 紫色 - 关键字（if, else, return, import...）
    val string = Color(0xFF98C379)         // 绿色 - 字符串字面量
    val comment = Color(0xFF5C6370)        // 灰蓝 - 注释
    val function = Color(0xFF61AFEF)       // 蓝色 - 函数名
    val functionBuiltin = Color(0xFF56B6C2) // 青色 - 内置函数
    val functionMethod = Color(0xFFD19A66)  // 橙色 - 方法名
    val type = Color(0xFFE5C07B)           // 黄色 - 类型名
    val number = Color(0xFFD19A66)         // 橙色 - 数字
    val operator = Color(0xFFABB2BF)       // 浅灰 - 运算符
    val identifier = Color(0xFFE06C75)     // 红色 - 标识符
    val variable = Color(0xFFE06C75)       // 红色 - 变量
    val property = Color(0xFFE06C75)       // 红色 - 属性
    val punctuation = Color(0xFFABB2BF)    // 浅灰 - 标点符号
    val escape = Color(0xFF56B6C2)         // 青色 - 转义字符
    val constantBuiltin = Color(0xFFE5C07B) // 黄色 - 内置常量（null, true, false...）
    val label = Color(0xFFE06C75)           // 红色 - 标签
    val namespace = Color(0xFFE5C07B)        // 黄色 - 命名空间
    val plainText = Color(0xFFABB2BF)        // 浅灰 - 纯文本默认颜色

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
        "identifier" to identifier,
        "variable" to variable,
        "property" to property,
        "punctuation" to punctuation,
        "escape" to escape,
        "constant.builtin" to constantBuiltin,
        "label" to label,
        "namespace" to namespace,
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