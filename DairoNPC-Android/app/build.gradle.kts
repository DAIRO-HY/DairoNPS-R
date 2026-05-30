plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.compose)
}

android {
    namespace = "cn.dairo.npc"
    compileSdk {
        version = release(36) {
            minorApiLevel = 1
        }
    }

    defaultConfig {
        applicationId = "cn.dairo.npc"
        minSdk = 23
        targetSdk = 36
        versionCode = 1
        versionName = "1.0.0"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

//    signingConfigs {
//        create("releaseConfig") {
//            storeFile = File(System.getenv("DAIRO_NPC_APK_JKS_PATH"))
//            storePassword = System.getenv("DAIRO_NPC_APK_JKS_PASSWORD")
//            keyAlias = System.getenv("DAIRO_NPC_APK_JKS_KEY_ALIAS")
//            keyPassword = System.getenv("DAIRO_NPC_APK_JKS_PASSWORD")
//        }
//    }
    buildTypes {
//        release {
//            signingConfig = signingConfigs.getByName("releaseConfig")
//            isMinifyEnabled = true
//            proguardFiles(
//                //在很多情况下，proguard-android-optimize.txt 文件并不是必须手动创建的。这个文件通常是由 Android Gradle 插件隐式引入的，用于启用一些激进的优化配置。
//                getDefaultProguardFile("proguard-android-optimize.txt"),
//                "proguard-rules.pro"
//            )
//        }
//        debug {
//            signingConfig = signingConfigs.getByName("releaseConfig")
//            isMinifyEnabled = false
//            proguardFiles(
//                //在很多情况下，proguard-android-optimize.txt 文件并不是必须手动创建的。这个文件通常是由 Android Gradle 插件隐式引入的，用于启用一些激进的优化配置。
//                getDefaultProguardFile("proguard-android-optimize.txt"),
//                "proguard-rules.pro"
//            )
//        }
    }
    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_16
        targetCompatibility = JavaVersion.VERSION_16
    }
    buildFeatures {
        compose = true
    }
}

dependencies {
    implementation(platform(libs.androidx.compose.bom))
    implementation(libs.androidx.activity.compose)
    implementation(libs.androidx.compose.material3)
    implementation(libs.androidx.compose.runtime)
    implementation(libs.androidx.compose.ui)
    implementation(libs.androidx.compose.ui.graphics)
    implementation(libs.androidx.compose.ui.tooling.preview)
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.lifecycle.runtime.ktx)
    implementation(libs.androidx.datastore)
    implementation(libs.androidx.lifecycle.viewmodel.compose)
    implementation(libs.androidx.navigation.compose)
    implementation(libs.androidx.material3)
    implementation(libs.androidx.compose.material.icons.extended)
    implementation(libs.gson)

    testImplementation(libs.junit)
    androidTestImplementation(platform(libs.androidx.compose.bom))
    androidTestImplementation(libs.androidx.compose.ui.test.junit4)
    androidTestImplementation(libs.androidx.espresso.core)
    androidTestImplementation(libs.androidx.junit)
    debugImplementation(libs.androidx.compose.ui.test.manifest)
    debugImplementation(libs.androidx.compose.ui.tooling)
}