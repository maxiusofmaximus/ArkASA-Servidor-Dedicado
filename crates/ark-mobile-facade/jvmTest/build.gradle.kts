// Host-JVM verification of the Rust <-> Kotlin bridge.
//
// This module exists to answer one question deterministically, on a developer
// machine, without the Android SDK, without an emulator and without a device:
// *do the UniFFI bindings generated from ark-core actually work?*
//
// `uniffi`'s Kotlin output loads the native library through JNA. On Windows
// that means the `ark_mobile_facade.dll` built by cargo, so the very same
// generated code that will later run on Android is exercised here against the
// real Rust implementation.
//
// Run with:  gradle test   (or the checked-in `gradlew`)
//
// The Rust side must have been built first:
//   cargo build -p ark-mobile-facade

plugins {
    kotlin("jvm") version "2.4.10"
}

repositories {
    mavenCentral()
}

dependencies {
    // The UniFFI-generated Kotlin sources are compiled into the MAIN source
    // set (see below); they need JNA, which is a runtime dependency of the
    // generated code, not of the test.
    implementation("net.java.dev.jna:jna:5.17.0")

    testImplementation(kotlin("test"))
    testImplementation("org.junit.jupiter:junit-jupiter:5.11.4")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

kotlin {
    jvmToolchain(17)
}

// Repository root. This module sits at
// <repo>/crates/ark-mobile-facade/jvmTest, so the repo root is THREE levels up.
// Resolved through the project dir itself (not `layout.projectDirectory.dir`)
// because the latter does not normalise `..` and silently yields no sources.
val repoRoot: File = projectDir.absoluteFile.parentFile.parentFile.parentFile

logger.lifecycle("ark bridge: repoRoot=$repoRoot")
logger.lifecycle("ark bridge: bindings=${File(repoRoot, "target/uniffi-bindings/kotlin").exists()}")

sourceSets {
    // Compile the generated bindings as production code so the tests exercise
    // exactly what an app would consume.
    main {
        kotlin.srcDir(File(repoRoot, "target/uniffi-bindings/kotlin"))
    }
}

tasks.test {
    useJUnitPlatform()

    // Point JNA at the freshly built Rust cdylib. Overridable so CI (and a
    // future macOS/Linux run) can pass its own directory and library name.
    val dllDir = providers.gradleProperty("arkFacadeLibDir")
        .orElse(File(repoRoot, "target/debug").absolutePath)
        .get()
    systemProperty("jna.library.path", dllDir)

    // Belt and braces: UniFFI reads this override before falling back to its
    // default library name resolution.
    systemProperty(
        "uniffi.component.ark_mobile_facade.libraryOverride",
        providers.gradleProperty("arkFacadeLibName").orElse("ark_mobile_facade").get()
    )

    testLogging {
        events("passed", "failed", "skipped")
        showStandardStreams = true
    }
}
