// The same plugin set macula-cam2me builds with, minus KSP (no Room here).
plugins {
    id("com.android.application") version "9.3.0" apply false
    id("org.jetbrains.kotlin.android") version "2.2.21" apply false
    // Tracks the Kotlin version 1:1.
    id("org.jetbrains.kotlin.plugin.compose") version "2.2.21" apply false
}
