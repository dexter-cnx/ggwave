plugins {
    id("com.android.application") version "9.0.0"
}

android {
    namespace = "io.github.dextercnx.ggwave.validation"
    compileSdk = 36

    defaultConfig {
        applicationId = "io.github.dextercnx.ggwave.validation"
        minSdk = 23
        targetSdk = 36
        versionCode = 1
        versionName = "1.0"
    }
}

dependencies {
    implementation(project(":ggwave-kotlin"))
}
