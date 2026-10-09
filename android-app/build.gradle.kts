// Top-level build for the Android companion app.
//
// AGP 8.7.3 is comfortably above the 8.5.1 floor that Android requires for
// 16 KB page-size devices, so the packaging of jniLibs is aligned by default.
plugins {
    id("com.android.application") version "8.7.3" apply false
    kotlin("android") version "2.4.10" apply false
    id("org.jetbrains.kotlin.plugin.compose") version "2.4.10" apply false
}
