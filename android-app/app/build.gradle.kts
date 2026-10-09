// Android-first companion app for the ARK ASA Configuration Manager.
//
// The pure domain logic lives in Rust (`ark-core`), exposed through
// `ark-mobile-facade` and consumed here through UniFFI-generated Kotlin. This
// module owns only the Android surface: no business rules are duplicated here.
//
// The native library is built by:
//   scripts/build-android.ps1   (wraps `cargo ndk`)
// which drops `libark_mobile_facade.so` into app/src/main/jniLibs/<abi>/.

plugins {
    id("com.android.application") version "8.7.3"
    kotlin("android") version "2.4.10"
    id("org.jetbrains.kotlin.plugin.compose") version "2.4.10"
}

android {
    namespace = "com.arkasa.mobile"
    compileSdk = 36

    defaultConfig {
        applicationId = "com.arkasa.mobile"
        minSdk = 26
        targetSdk = 36
        versionCode = 1
        versionName = "2.1.0"
    }

    buildTypes {
        release {
            // Keep artefacts small; the Rust library is stripped separately.
            isMinifyEnabled = true
            isShrinkResources = true
            // Signed with the debug key so it can be installed on a personal
            // device today. A real release keystore must replace this before any
            // store upload (see Phase 5 in docs/MOBILE_DECISION.md).
            signingConfig = signingConfigs.getByName("debug")
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlin {
        jvmToolchain(17)
    }

    // The UniFFI bindings are generated from the built cdylib into
    // target/uniffi-bindings/kotlin and compiled as ordinary Kotlin here.
    sourceSets {
        getByName("main") {
            kotlin.srcDir(
                File(
                    rootProject.projectDir.parentFile,
                    "target/uniffi-bindings/kotlin"
                )
            )
        }
    }

    buildFeatures {
        compose = true
    }

    packaging {
        jniLibs {
            // UniFFI resolves the library by name; keep it uncompressed and
            // 16 KB aligned (AGP >= 8.5.1 handles the alignment).
            useLegacyPackaging = false
        }
    }
}

dependencies {
    // JNA is required by the UniFFI-generated Kotlin at runtime.
    implementation("net.java.dev.jna:jna:5.17.0@aar")

    implementation("androidx.core:core-ktx:1.15.0")
    implementation("androidx.activity:activity-compose:1.9.3")

    val composeBom = platform("androidx.compose:compose-bom:2024.12.01")
    implementation(composeBom)
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.compose.material:material-icons-extended")
    implementation("androidx.lifecycle:lifecycle-viewmodel-compose:2.8.7")
    implementation("androidx.lifecycle:lifecycle-runtime-compose:2.8.7")
    implementation("androidx.navigation:navigation-compose:2.8.5")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.9.0")

    debugImplementation("androidx.compose.ui:ui-tooling")

    testImplementation("junit:junit:4.13.2")
}
