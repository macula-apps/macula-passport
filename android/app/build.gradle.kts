plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("org.jetbrains.kotlin.plugin.compose")
}

// The release version comes from the v* tag (release.yml passes -PversionName);
// the code orders installs, so 1.2.3 is 10203 and every later tag installs over
// an earlier one.
val releaseVersionName = providers.gradleProperty("versionName").getOrElse("0.1.0")
val releaseVersionCode = releaseVersionName.split(".").map { it.toInt() }
    .let { (major, minor, patch) -> major * 10000 + minor * 100 + patch }

// The release key lives only in CI secrets (#6); release.yml writes the
// keystore to the runner and names it here. Without these the release apk is
// built unsigned, which no phone installs.
val releaseStoreFile = System.getenv("MACULA_PASSPORT_STORE_FILE")
val releaseStorePassword = System.getenv("MACULA_PASSPORT_STORE_PASSWORD")

android {
    namespace = "io.macula.passport"
    // compose-bom's transitive deps need API 37 to compile against.
    compileSdk = 37

    defaultConfig {
        applicationId = "io.macula.passport"
        minSdk = 26
        targetSdk = 37
        versionCode = releaseVersionCode
        versionName = releaseVersionName
        // The two ABIs scripts/build_android_libs.sh builds: phones and the
        // CI emulator.
        ndk {
            abiFilters += listOf("arm64-v8a", "x86_64")
        }
    }

    signingConfigs {
        if (releaseStoreFile != null && releaseStorePassword != null) {
            create("release") {
                storeFile = file(releaseStoreFile)
                storePassword = releaseStorePassword
                keyAlias = "macula-passport"
                keyPassword = releaseStorePassword
            }
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            signingConfig = signingConfigs.findByName("release")
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_21
        targetCompatibility = JavaVersion.VERSION_21
    }

    // jniLibs/ and the generated bindings under kotlin/uniffi/ come from
    // ../../scripts/build_android_libs.sh; neither is committed.
    sourceSets {
        getByName("main") {
            jniLibs.srcDirs("src/main/jniLibs")
        }
    }

    lint {
        // The generated bindings call java.lang.ref.Cleaner (API 33) only
        // behind a runtime class check with a JNA fallback below it; Lint
        // cannot see through that guard.
        disable += "NewApi"
    }
}

dependencies {
    implementation("androidx.core:core-ktx:1.19.0")
    implementation("androidx.activity:activity-compose:1.13.0")
    implementation(platform("androidx.compose:compose-bom:2026.08.00"))
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.compose.material:material-icons-core")
    implementation("androidx.lifecycle:lifecycle-runtime-ktx:2.10.0")

    // Runtime needs of the generated UniFFI bindings.
    implementation("net.java.dev.jna:jna:5.17.0@aar")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.10.2")

    testImplementation("junit:junit:4.13.2")
}
