package com.example.collisions.ViewModels

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import com.example.collisions.Models.IArtifact
import com.example.collisions.Models.LocalPayload
import com.example.collisions.Processing.CodeParseResult
import com.example.collisions.Processing.FileProcessor
import com.example.collisions.Repositories.LocalArtifactRepo
import com.example.collisions.Repositories.LocalFileSystem
import com.example.collisions.Utils.FormatSize
import kotlinx.coroutines.*

class MainViewModel(
    private val fs: LocalFileSystem,
    private val repo: LocalArtifactRepo,
) {
    companion object {
        private const val WIDE_MODE_THRESHOLD = 640
    }

    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
    private val childrenCache = mutableMapOf<String, List<IArtifact>>()

    // 文件浏览状态
    var currentPath by mutableStateOf("")
        private set

    var totalSize by mutableStateOf(0L)
        private set

    var isComputingSize by mutableStateOf(false)
        private set

    var selectedArtifact by mutableStateOf<IArtifact?>(null)
        private set

    var selectedContent by mutableStateOf<String?>(null)
        private set

    var messageText by mutableStateOf<String?>(null)
        private set

    var selectedParseResult by mutableStateOf<CodeParseResult?>(null)
        private set

    var treeItems by mutableStateOf<List<TreeItemViewModel>>(emptyList())
        private set

    var hasWorkspace by mutableStateOf(false)
        private set

    var hasSelection by mutableStateOf(false)
        private set

    var isDrawerOpen by mutableStateOf(false)
        private set

    var isWide by mutableStateOf(true)
        private set

    val totalSizeReadable: String get() = FormatSize.readable(totalSize)
    val selectedSizeDisplay: String get() = selectedArtifact?.let { FormatSize.readable(it.size) } ?: ""
    val isCodePreviewVisible: Boolean get() = hasSelection && messageText == null

    // 平台相关的文件选择器注入
    var pickFolderAction: (suspend () -> String?)? = null

    fun pickFolder() {
        scope.launch {
            val path = pickFolderAction?.invoke()
            if (path != null) {
                loadCore(path)
            }
        }
    }

    fun closeWorkspace() {
        currentPath = ""
        hasWorkspace = false
        childrenCache.clear()
        treeItems = emptyList()
        totalSize = 0
        selectedArtifact = null
        selectedContent = null
        selectedParseResult = null
        hasSelection = false
        messageText = null
    }

    fun selectItem(item: TreeItemViewModel?) {
        if (item == null) return
        if (item.isDir) {
            item.toggleExpanded()
        } else {
            selectFile(item.artifact)
        }
    }

    fun expandAll() {
        for (root in treeItems) {
            root.expandAllRecursive()
        }
    }

    fun collapseAll() {
        for (root in treeItems) {
            root.collapseRecursive()
        }
    }

    fun clearSelection() {
        selectedArtifact = null
        hasSelection = false
        selectedContent = null
        selectedParseResult = null
        messageText = null
    }

    fun toggleDrawer() {
        isDrawerOpen = !isDrawerOpen
    }

    fun onWindowResized(width: Double) {
        isWide = width > WIDE_MODE_THRESHOLD
        if (isWide) {
            isDrawerOpen = false
        }
    }

    fun loadCore(path: String) {
        scope.launch {
            currentPath = path
            hasWorkspace = true
            childrenCache.clear()
            totalSize = 0
            treeItems = emptyList()

            try {
                val listResult = repo.listAsync(path)
                val items = listResult.getOrNull() ?: emptyList()
                treeItems = items.map { TreeItemViewModel(it, repo, childrenCache) }
            } catch (ex: Exception) {
                messageText = "加载失败: ${ex.message}"
            }

            // 异步计算总大小
            isComputingSize = true
            launch {
                val size = computeTotalSize(path)
                totalSize = size
                isComputingSize = false
            }
        }
    }

    private fun selectFile(artifact: IArtifact) {
        scope.launch {
            // 先清除所有旧状态，避免白屏
            messageText = null
            selectedContent = null
            selectedParseResult = null
            selectedArtifact = artifact
            hasSelection = true

            // 处理文件
            processFile(artifact)
        }
    }

    private suspend fun processFile(artifact: IArtifact) {
        // 检查是否是目录
        if (artifact.payload is LocalPayload && (artifact.payload as LocalPayload).isDir) {
            return
        }

        val path = artifact.id
        if (path.isEmpty()) {
            messageText = "无法读取文件: ${artifact.name}"
            return
        }

        // 检查是否是文本文件
        if (!fs.isTextFile(path)) {
            messageText = "[二进制文件] ${artifact.name} 无法预览"
            return
        }

        // 读取文件内容
        val contentResult = repo.tryReadTextAsync(path)
        val content = contentResult.getOrNull()
        if (content == null) {
            messageText = "无法读取文件: ${artifact.name}"
            return
        }

        val normalizedContent = content.replace("\t", "    ")
        val ext = artifact.extension
        val filename = artifact.name

        // 解析代码
        val parseResult = try {
            FileProcessor.process(normalizedContent, ext, filename)
        } catch (ex: Exception) {
            println("Code parsing failed for ${artifact.name}: ${ex.message}")
            null
        }

        selectedParseResult = parseResult
        selectedContent = normalizedContent
    }

    private suspend fun computeTotalSize(path: String): Long {
        return withContext(Dispatchers.IO) {
            try {
                val items = repo.listAsync(path)
                val artifacts = items.getOrNull() ?: return@withContext 0L

                val deferredResults = artifacts.map { item ->
                    async {
                        if (item.payload is LocalPayload && (item.payload as LocalPayload).isDir) {
                            val payload = item.payload as LocalPayload
                            computeTotalSize(payload.absolutePath)
                        } else {
                            item.size
                        }
                    }
                }
                deferredResults.sumOf { it.await() }
            } catch (e: CancellationException) {
                println("computeTotalSize canceled for path: $path")
                0L
            } catch (e: Exception) {
                println("computeTotalSize error for path: $path, message: ${e.message}")
                0L
            }
        }
    }

    fun dispose() {
        scope.cancel()
    }
}