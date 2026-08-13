package com.example.collisions.Processing

import kotlin.test.Test
import kotlin.test.assertTrue
import kotlin.test.assertFalse
import kotlin.test.assertEquals

class CodeParserJvmTest {

    @Test
    fun `parse Python code returns highlights`() {
        val source = """
            def greet(name: str) -> str:
                x = 42
                return f"Hello, {name}"
        """.trimIndent()

        // Rust 端 parse_code 需要带点号的扩展名（如 .py）
        val result = parseCode(source, ".py")

        assertTrue(result is CodeParseResult.Code, "Expected Code result, got $result")
        if (result is CodeParseResult.Code) {
            assertTrue(result.highlightsByLine.isNotEmpty(), "Expected highlights")
            assertTrue(result.highlightsByLine.flatten().isNotEmpty(), "Expected highlight tokens")
            // 验证 kind 映射正确
            val kinds = result.highlightsByLine.flatten().map { it.kind }.toSet()
            assertTrue(kinds.isNotEmpty(), "Expected kinds")
            // 确保没有映射失败的 SCREAMING_SNAKE_CASE
            assertFalse(kinds.any { it.contains("_") && it == it.uppercase() }, "Found unmapped kinds: $kinds")
        }
    }

    @Test
    fun `parse Python code returns outline`() {
        val source = """
            class Foo:
                def bar(self) -> None:
                    pass
                def baz(self) -> None:
                    pass
        """.trimIndent()

        val result = parseCode(source, ".py")

        assertTrue(result is CodeParseResult.Code, "Expected Code result, got $result")
        if (result is CodeParseResult.Code) {
            assertTrue(result.outline.isNotEmpty(), "Expected outline nodes")
            // 验证 outline 有 class Foo
            val hasClass = result.outline.any { it.name == "Foo" }
            assertTrue(hasClass, "Expected class Foo in outline, got: ${result.outline}")
            // 验证 outline 有方法 bar（递归搜索子节点）
            fun findByName(nodes: List<OutlineNode>, name: String): Boolean =
                nodes.any { it.name == name || findByName(it.children, name) }
            assertTrue(findByName(result.outline, "bar"), "Expected method bar in outline: ${result.outline}")
        }
    }

    @Test
    fun `plain text returns PlainText result`() {
        val result = parseCode("just some text", "txt")
        assertTrue(result is CodeParseResult.PlainText, "Expected PlainText, got $result")
    }

    @Test
    fun `kind mapping works correctly`() {
        assertEquals("keyword", HighlightToken.mapKind("KEYWORD"))
        assertEquals("string", HighlightToken.mapKind("STRING_LITERAL"))
        assertEquals("comment", HighlightToken.mapKind("COMMENT"))
        assertEquals("function", HighlightToken.mapKind("FUNCTION"))
        assertEquals("function.builtin", HighlightToken.mapKind("FUNCTION_BUILTIN"))
        assertEquals("function.method", HighlightToken.mapKind("FUNCTION_METHOD"))
        assertEquals("type", HighlightToken.mapKind("TYPE"))
        assertEquals("number", HighlightToken.mapKind("NUMBER"))
        assertEquals("operator", HighlightToken.mapKind("OPERATOR"))
        assertEquals("identifier", HighlightToken.mapKind("IDENTIFIER"))
        assertEquals("variable", HighlightToken.mapKind("VARIABLE"))
        assertEquals("property", HighlightToken.mapKind("PROPERTY"))
        assertEquals("punctuation", HighlightToken.mapKind("PUNCTUATION"))
        assertEquals("escape", HighlightToken.mapKind("ESCAPE"))
        assertEquals("constant.builtin", HighlightToken.mapKind("CONSTANT_BUILTIN"))
        assertEquals("label", HighlightToken.mapKind("LABEL"))
        assertEquals("namespace", HighlightToken.mapKind("NAMESPACE"))
        assertEquals("identifier", HighlightToken.mapKind("UNKNOWN"))
    }
}