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
    fun `unknown extension returns Code with empty highlights`() {
        val result = parseCode("just some text", ".unknown")
        assertTrue(result is CodeParseResult.Code, "Expected Code, got $result")
        if (result is CodeParseResult.Code) {
            assertTrue(result.highlightsByLine.all { it.isEmpty() }, "Expected empty highlights for unknown extension")
        }
    }

    @Test
    fun `parse Go code returns highlights`() {
        val source = """
            package main

            import "fmt"

            func main() {
                fmt.Println("hello world")
            }
        """.trimIndent()

        val result = parseCode(source, ".go")

        assertTrue(result is CodeParseResult.Code, "Expected Code result, got $result")
        if (result is CodeParseResult.Code) {
            assertTrue(result.highlightsByLine.isNotEmpty(), "Expected highlights for Go")
            val kinds = result.highlightsByLine.flatten().map { it.kind }.toSet()
            assertTrue(kinds.isNotEmpty(), "Expected kinds for Go")
            assertFalse(kinds.any { it.contains("_") && it == it.uppercase() }, "Found unmapped kinds: $kinds")
        }
    }

    @Test
    fun `parse JavaScript code returns highlights`() {
        val source = """
            function greet(name) {
                const x = 42;
                return `Hello, ${'$'}name`;
            }
        """.trimIndent()

        val result = parseCode(source, ".js")

        assertTrue(result is CodeParseResult.Code, "Expected Code result, got $result")
        if (result is CodeParseResult.Code) {
            assertTrue(result.highlightsByLine.isNotEmpty(), "Expected highlights for JavaScript")
            val kinds = result.highlightsByLine.flatten().map { it.kind }.toSet()
            assertTrue(kinds.isNotEmpty(), "Expected kinds for JavaScript")
            assertFalse(kinds.any { it.contains("_") && it == it.uppercase() }, "Found unmapped kinds: $kinds")
        }
    }

    @Test
    fun `parse Rust code returns highlights`() {
        val source = """
            fn main() {
                let x = 42;
                println!("hello world");
            }
        """.trimIndent()

        val result = parseCode(source, ".rs")

        assertTrue(result is CodeParseResult.Code, "Expected Code result, got $result")
        if (result is CodeParseResult.Code) {
            assertTrue(result.highlightsByLine.isNotEmpty(), "Expected highlights for Rust")
            val kinds = result.highlightsByLine.flatten().map { it.kind }.toSet()
            assertTrue(kinds.isNotEmpty(), "Expected kinds for Rust")
            assertFalse(kinds.any { it.contains("_") && it == it.uppercase() }, "Found unmapped kinds: $kinds")
        }
    }

    @Test
    fun `parse C code returns highlights`() {
        val source = """
            #include <stdio.h>

            int main() {
                printf("hello world");
                return 0;
            }
        """.trimIndent()

        val result = parseCode(source, ".c")

        assertTrue(result is CodeParseResult.Code, "Expected Code result, got $result")
        if (result is CodeParseResult.Code) {
            assertTrue(result.highlightsByLine.isNotEmpty(), "Expected highlights for C")
            val kinds = result.highlightsByLine.flatten().map { it.kind }.toSet()
            assertTrue(kinds.isNotEmpty(), "Expected kinds for C")
            assertFalse(kinds.any { it.contains("_") && it == it.uppercase() }, "Found unmapped kinds: $kinds")
        }
    }

    @Test
    fun `parse Java code returns highlights`() {
        val source = """
            class Hello {
                public static void main(String[] args) {
                    System.out.println("hello");
                }
            }
        """.trimIndent()

        val result = parseCode(source, ".java")

        assertTrue(result is CodeParseResult.Code, "Expected Code result, got $result")
        if (result is CodeParseResult.Code) {
            assertTrue(result.highlightsByLine.isNotEmpty(), "Expected highlights for Java")
            val kinds = result.highlightsByLine.flatten().map { it.kind }.toSet()
            assertTrue(kinds.isNotEmpty(), "Expected kinds for Java")
            assertFalse(kinds.any { it.contains("_") && it == it.uppercase() }, "Found unmapped kinds: $kinds")
        }
    }

    @Test
    fun `parse TypeScript code returns highlights`() {
        val source = """
            function greet(name: string): void {
                const x: number = 42;
            }
        """.trimIndent()

        val result = parseCode(source, ".ts")

        assertTrue(result is CodeParseResult.Code, "Expected Code result, got $result")
        if (result is CodeParseResult.Code) {
            assertTrue(result.highlightsByLine.isNotEmpty(), "Expected highlights for TypeScript")
            val kinds = result.highlightsByLine.flatten().map { it.kind }.toSet()
            assertTrue(kinds.isNotEmpty(), "Expected kinds for TypeScript")
            assertFalse(kinds.any { it.contains("_") && it == it.uppercase() }, "Found unmapped kinds: $kinds")
        }
    }

    @Test
    fun `unsupported extension returns empty highlights`() {
        val source = "some unknown code"
        val result = parseCode(source, ".unknown")

        assertTrue(result is CodeParseResult.Code, "Expected Code result, got $result")
        if (result is CodeParseResult.Code) {
            assertTrue(result.highlightsByLine.all { it.isEmpty() }, "Expected empty highlights for unsupported extension")
        }
    }

    @Test
    fun `parse C code with line and block comments returns Comment tokens`() {
        val source = """
            int main() {
                // line comment
                /* block comment */
                return 0;
            }
        """.trimIndent()

        val result = parseCode(source, ".c")

        assertTrue(result is CodeParseResult.Code, "Expected Code result, got $result")
        if (result is CodeParseResult.Code) {
            val kinds = result.highlightsByLine.flatten().map { it.kind }.toSet()
            assertTrue(
                "comment" in kinds,
                "Expected comment tokens for // and /* */, got kinds: $kinds"
            )
        }
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