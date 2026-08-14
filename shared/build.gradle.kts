import org.jetbrains.kotlin.gradle.dsl.JvmTarget

// Rust native 库路径
val nativeProjectDir = rootProject.file("native")
val nativeTargetDir = nativeProjectDir.resolve("target")
val nativeLibName = "uniffi_code_parser"
val jvmNativeLib = nativeTargetDir.resolve("debug/lib${nativeLibName}.dylib")
val uniffiKotlinOutDir = layout.buildDirectory.dir("generated/uniffi/kotlin")

// 构建 Rust 库 (debug)
val cargoBuildJvm by tasks.registering(Exec::class) {
    group = "uniffi"
    description = "Build Rust native library for JVM (debug)"
    workingDir = nativeProjectDir
    commandLine("cargo", "build")
    inputs.dir(nativeProjectDir.resolve("src"))
    inputs.file(nativeProjectDir.resolve("Cargo.toml"))
    outputs.file(jvmNativeLib)
}

// 复制 JVM 原生库到 build 输出目录
val jvmNativeLibOutputDir = layout.buildDirectory.dir("nativeLibs")
val copyJvmNativeLib by tasks.registering(Copy::class) {
    group = "uniffi"
    description = "Copy JVM native library to build output"
    dependsOn(cargoBuildJvm)
    from(jvmNativeLib)
    into(jvmNativeLibOutputDir)
}

// 生成 uniffi Kotlin 绑定（JNA）
// uniffi 生成一个 JNA 绑定文件，可供 jvmMain 和 androidMain 共用
val generateUniffiKotlinBindings by tasks.registering(Exec::class) {
    group = "uniffi"
    description = "Generate Kotlin Multiplatform bindings from Rust library using uniffi-bindgen"
    workingDir = nativeProjectDir
    val outDir = uniffiKotlinOutDir.get().asFile
    val nativeLibPath = jvmNativeLib.absolutePath
    outputs.dir(outDir)
    doFirst {
        // cargo构建仅在构建时触发
        if (!File(nativeLibPath).exists()) {
            throw GradleException(
                "Rust native library not found: $nativeLibPath\n" +
                    "Please run './gradlew build' first to build the Rust library."
            )
        }
        outDir.mkdirs()
    }
    commandLine(
        "uniffi-bindgen", "generate",
        "--library", nativeLibPath,
        "--language", "kotlin",
        "--out-dir", outDir.absolutePath
    )
}

plugins {
    alias(libs.plugins.kotlinMultiplatform)
    alias(libs.plugins.androidMultiplatformLibrary)
    alias(libs.plugins.composeMultiplatform)
    alias(libs.plugins.composeCompiler)
}

kotlin {
    jvm()

    android {
        namespace = "com.example.collisions.shared"
        compileSdk = libs.versions.android.compileSdk.get().toInt()
        minSdk = libs.versions.android.minSdk.get().toInt()

        compilerOptions {
            jvmTarget = JvmTarget.JVM_11
        }
        androidResources {
            enable = true
        }
        withHostTest {
            isIncludeAndroidResources = true
        }
        withDeviceTestBuilder {
            sourceSetTreeName = "test"
        }.configure {
            instrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        }
    }

    sourceSets {
        val generatedDir = uniffiKotlinOutDir.get().asFile

        commonMain {
            dependencies {
                implementation(libs.compose.runtime)
                implementation(libs.compose.foundation)
                implementation(libs.compose.material3)
                implementation(libs.compose.ui)
                implementation(libs.compose.components.resources)
                implementation(libs.compose.uiToolingPreview)
                implementation(libs.androidx.lifecycle.viewmodelCompose)
                implementation(libs.androidx.lifecycle.runtimeCompose)
                // Material Icons（像 Flutter 的 Icons.*）
                implementation(libs.compose.materialIconsCore)
                implementation(libs.compose.materialIconsExtended)
            }
        }
        jvmMain {
            // uniffi 生成的 JNA 绑定（jvmMain 和 androidMain 共用同一份）
            kotlin.srcDir(generatedDir)
            dependencies {
                implementation(libs.jna)
                implementation(libs.jna.platform)
            }
        }
        androidMain {
            // uniffi 生成的 JNA 绑定（jvmMain 和 androidMain 共用同一份）
            kotlin.srcDir(generatedDir)
            dependencies {
                implementation(libs.jna)
                implementation(libs.jna.platform)
                implementation(libs.compose.uiToolingPreview)
            }
        }
        commonTest.dependencies {
            implementation(libs.kotlin.test)
        }
    }
}

// 让 jvm 编译任务依赖 uniffi 绑定生成
tasks.named("compileKotlinJvm") {
    dependsOn(generateUniffiKotlinBindings)
}

// 让 jvmTest 任务依赖 uniffi 绑定生成
tasks.named("compileTestKotlinJvm") {
    dependsOn(generateUniffiKotlinBindings)
}

// jvm 测试运行参数：加载原生库
tasks.withType<Test>().configureEach {
    dependsOn(cargoBuildJvm)
    // uniffi 通过 libraryOverride 属性加载原生库（绝对路径）
    systemProperty("uniffi.component.$nativeLibName.libraryOverride", jvmNativeLib.absolutePath)
    // JNA 兜底
    systemProperty("java.library.path", jvmNativeLib.parentFile.absolutePath)
}

// assemble 时自动编译 Rust 并复制原生库到 build/nativeLibs/
tasks.named("assemble") {
    dependsOn(copyJvmNativeLib)
}

