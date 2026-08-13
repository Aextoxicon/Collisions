package com.example.collisions

import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import javax.swing.JFileChooser

fun main() = application {
    Window(
        onCloseRequest = ::exitApplication,
        title = "Collisions",
    ) {
        App(
            pickFolderAction = {
                withContext(Dispatchers.IO) {
                    val chooser = JFileChooser()
                    chooser.fileSelectionMode = JFileChooser.DIRECTORIES_ONLY
                    chooser.isAcceptAllFileFilterUsed = false
                    val result = chooser.showOpenDialog(null)
                    if (result == JFileChooser.APPROVE_OPTION) {
                        chooser.selectedFile?.absolutePath
                    } else {
                        null
                    }
                }
            },
        )
    }
}
