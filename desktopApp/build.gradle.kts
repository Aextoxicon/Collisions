import org.jetbrains.compose.desktop.application.dsl.TargetFormat

plugins {
    alias(libs.plugins.kotlinJvm)
    alias(libs.plugins.composeMultiplatform)
    alias(libs.plugins.composeCompiler)
}

val nativeLibDir = rootProject.file("native/target/debug").absolutePath

dependencies {
    implementation(project(":shared"))

    implementation(compose.desktop.currentOs)
    implementation(libs.kotlinx.coroutinesSwing)

    implementation(libs.compose.uiToolingPreview)
}

compose.desktop {
    application {
        mainClass = "com.example.collisions.MainKt"

        nativeDistributions {
            targetFormats(TargetFormat.Dmg, TargetFormat.Msi, TargetFormat.Deb)
            packageName = "com.example.collisions"
            packageVersion = "1.0.0"
        }
    }
}

tasks.withType<JavaExec>().configureEach {
    systemProperty("java.library.path", nativeLibDir)
    environment("DYLD_LIBRARY_PATH", nativeLibDir)
}
