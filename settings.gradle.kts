rootProject.name = "Clash Multiplatform Compat"

include(":base")
include(":jni")

dependencyResolutionManagement {
    repositories {
        mavenCentral()
    }
    versionCatalogs {
        create("libs") {
            library("apache-compress-core", "org.apache.commons:commons-compress:1.23.0")
            library("apache-compress-zstd", "com.github.luben:zstd-jni:1.5.4-2")
            bundle("apache-compress", listOf("apache-compress-core", "apache-compress-zstd"))
        }
    }
}
