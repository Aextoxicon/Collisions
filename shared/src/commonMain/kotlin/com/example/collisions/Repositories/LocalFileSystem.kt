package com.example.collisions.Repositories

data class LocalFileInfo(
    val path: String,
    val name: String,
    val parentPath: String,
    val isDir: Boolean,
    val size: Long,
    val lastMod: Long,
    val extension: String,
)

expect class LocalFileSystem() {
    fun listFiles(path: String): List<LocalFileInfo>
    fun fileInfo(path: String): LocalFileInfo
    fun isTextFile(path: String): Boolean
    fun tryReadText(path: String): String?
    fun delete(path: String): Boolean
    fun toUri(path: String): String
}