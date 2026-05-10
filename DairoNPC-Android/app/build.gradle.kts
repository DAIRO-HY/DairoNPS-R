import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("kotlin-kapt")
}


android {
    namespace = "cn.dairo.npc"
    compileSdk = 36

    defaultConfig {
        applicationId = "cn.dairo.npc"
        minSdk = 29
        targetSdk = 36
        versionCode = 1
        versionName = "1.0.0"

        vectorDrawables {
            useSupportLibrary = true
        }
    }
    dataBinding {
        enable = false
    }
    signingConfigs {
        create("releaseConfig") {
            storeFile = File(rootProject.projectDir, "farming.jks")
            storePassword = "Hd78.jioigp-puUInn"
            keyAlias = "farming"
            //keyPassword = System.getenv("KEY_PASSWORD")
            keyPassword = "Hd78.jioigp-puUInn"
        }
    }
    buildTypes {
        release {
            signingConfig = signingConfigs.getByName("releaseConfig")

            //开启代码混淆
            isMinifyEnabled = true
            proguardFiles(
                //在很多情况下，proguard-android-optimize.txt 文件并不是必须手动创建的。这个文件通常是由 Android Gradle 插件隐式引入的，用于启用一些激进的优化配置。
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
        debug {
            signingConfig = signingConfigs.getByName("releaseConfig")

            //开启代码混淆
            isMinifyEnabled = false
            proguardFiles(
                //在很多情况下，proguard-android-optimize.txt 文件并不是必须手动创建的。这个文件通常是由 Android Gradle 插件隐式引入的，用于启用一些激进的优化配置。
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
        compilerOptions {
            jvmTarget.set(JvmTarget.JVM_17)
        }
    }
    buildFeatures {
        compose = false//禁用Compose,该项目没有使用Compose,这里要明确禁用,否则会报错
        buildConfig = true
    }
    packaging {
        resources {
            excludes += "/META-INF/{AL2.0,LGPL2.1}"
        }
    }
}

dependencies {
}