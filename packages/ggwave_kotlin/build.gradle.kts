plugins {
    id("com.android.library") version "9.0.0"
    id("maven-publish")
}

android {
    namespace = "io.github.dextercnx.ggwave"
    compileSdk = 36

    defaultConfig {
        minSdk = 23
        consumerProguardFiles("consumer-rules.pro")
    }

    publishing {
        singleVariant("release") {
            withSourcesJar()
        }
    }
}

group = "io.github.dextercnx"
version = "1.2.0"

publishing {
    publications {
        register<MavenPublication>("release") {
            groupId = project.group.toString()
            artifactId = "ggwave-kotlin"
            version = project.version.toString()
            afterEvaluate {
                from(components["release"])
            }
            pom {
                name.set("ggwave-kotlin")
                description.set("Kotlin/Android bindings for the universal ggwave Rust core.")
                url.set("https://github.com/dexter-cnx/ggwave")
                licenses {
                    license {
                        name.set("MIT License")
                        url.set("https://opensource.org/license/mit")
                    }
                }
                scm {
                    url.set("https://github.com/dexter-cnx/ggwave")
                    connection.set("scm:git:https://github.com/dexter-cnx/ggwave.git")
                }
            }
        }
    }
}
