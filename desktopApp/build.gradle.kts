import org.jetbrains.compose.desktop.application.dsl.TargetFormat

plugins {
    alias(libs.plugins.kotlinJvm)
    alias(libs.plugins.composeMultiplatform)
    alias(libs.plugins.composeCompiler)
}

val nativeLibDir = layout.buildDirectory.dir("nativeLibs/jvm")
val copyDesktopNativeLib by tasks.registering(Copy::class) {
    dependsOn(":shared:copyJvmNativeLib")
    from(project(":shared").layout.buildDirectory.dir("nativeLibs/jvm"))
    into(nativeLibDir.map { it.dir("common") })
}

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
            targetFormats(TargetFormat.Dmg, TargetFormat.Msi, TargetFormat.Deb, TargetFormat.Exe)
            packageName = "com.example.collisions"
            packageVersion = "1.0.0"
            appResourcesRootDir.set(nativeLibDir)
        }
    }
}

tasks.matching {
    it.name == "createDistributable" ||
        it.name.startsWith("package") ||
        it.name == "prepareAppResources"
}.configureEach {
    dependsOn(copyDesktopNativeLib)
}

tasks.withType<JavaExec>().configureEach {
    dependsOn(copyDesktopNativeLib)
    systemProperty("java.library.path", nativeLibDir.get().asFile.absolutePath)
    environment("DYLD_LIBRARY_PATH", nativeLibDir.get().asFile.absolutePath)
}