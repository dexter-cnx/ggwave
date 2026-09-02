pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "ggwave-kotlin-validation"
include(":app", ":ggwave-kotlin")
project(":ggwave-kotlin").projectDir = file("../../packages/ggwave_kotlin")
