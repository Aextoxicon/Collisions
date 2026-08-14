import org.jetbrains.kotlin.gradle.dsl.JvmTarget

val nativeProjectDir = rootProject.file("native")
val nativeTargetDir = nativeProjectDir.resolve("target")
val nativeLibName = "uniffi_code_parser"
val jvmNativeLib = nativeTargetDir.resolve("debug/lib${nativeLibName}.dylib")
val uniffiKotlinOutDir = layout.buildDirectory.dir("generated/uniffi/kotlin")

val cargoBuildJvm by tasks.registering(Exec::class) {
    group = "uniffi"
    description = "Build Rust native library for JVM (debug)"
    workingDir = nativeProjectDir
    commandLine("cargo", "build")
    inputs.dir(nativeProjectDir.resolve("src"))
    inputs.file(nativeProjectDir.resolve("Cargo.toml"))
    outputs.file(jvmNativeLib)
}

// 复制 JVM 原生库到build输出目录
val jvmNativeLibOutputDir = layout.buildDirectory.dir("nativeLibs")
val copyJvmNativeLib by tasks.registering(Copy::class) {
    group = "uniffi"
    description = "Copy JVM native library to build output"
    dependsOn(cargoBuildJvm)
    from(jvmNativeLib)
    into(jvmNativeLibOutputDir)
}

// jvm和android共用
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
                implementation(libs.compose.materialIconsCore)
                implementation(libs.compose.materialIconsExtended)
            }
        }
        jvmMain {
            kotlin.srcDir(generatedDir)
            dependencies {
                implementation(libs.jna)
                implementation(libs.jna.platform)
            }
        }
        androidMain {
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

tasks.named("compileKotlinJvm") {
    dependsOn(generateUniffiKotlinBindings)
}

tasks.named("compileAndroidMain") {
    dependsOn(generateUniffiKotlinBindings)
}

tasks.named("compileTestKotlinJvm") {
    dependsOn(generateUniffiKotlinBindings)
}

tasks.withType<Test>().configureEach {
    dependsOn(cargoBuildJvm)
    systemProperty("uniffi.component.$nativeLibName.libraryOverride", jvmNativeLib.absolutePath)
    systemProperty("java.library.path", jvmNativeLib.parentFile.absolutePath)
}

tasks.named("assemble") {
    dependsOn(copyJvmNativeLib)
}

