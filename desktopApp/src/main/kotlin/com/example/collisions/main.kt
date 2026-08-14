package com.example.collisions

import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.awt.FileDialog
import javax.swing.JFileChooser
import javax.swing.UIManager

fun main() = application {
    Window(
        onCloseRequest = ::exitApplication,
        title = "Collisions",
    ) {
        App(
            pickFolderAction = {
                withContext(Dispatchers.IO) {
                    pickFolderNative()
                }
            },
        )
    }
}

/** macOS: 使用 AWT FileDialog，调用NSOpenPanel
    Windows/Linux: JFileChooser+系统 L&F
 */
private fun pickFolderNative(): String? {
    val osName = System.getProperty("os.name").lowercase()
    return if (osName.contains("mac")) {
        pickFolderMac()
    } else {
        pickFolderFallback()
    }
}

private fun pickFolderMac(): String? {
    System.setProperty("apple.awt.fileDialogForDirectories", "true")
    val dialog = FileDialog(null as java.awt.Frame?, "选择文件夹", FileDialog.LOAD)
    dialog.isVisible = true // 阻塞，弹出真正的原生 NSOpenPanel
    return dialog.directory?.let { dir ->
        dialog.file?.let { "${dir}${it}" } ?: dir
    }
}

private fun pickFolderFallback(): String? {
    try {
        UIManager.setLookAndFeel(UIManager.getSystemLookAndFeelClassName())
    } catch (_: Exception) {}
    val chooser = JFileChooser().apply {
        fileSelectionMode = JFileChooser.DIRECTORIES_ONLY
        isAcceptAllFileFilterUsed = false
    }
    return if (chooser.showOpenDialog(null) == JFileChooser.APPROVE_OPTION) {
        chooser.selectedFile?.absolutePath
    } else null
}
