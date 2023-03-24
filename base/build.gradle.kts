plugins {
    java
    `maven-publish`
}

dependencies {
    compileOnly("org.jetbrains:annotations:24.0.1")
}

java {
    withSourcesJar()
}

publishing {
    publications {
        create("compat", MavenPublication::class) {
            artifactId = "compat"

            from(components["java"])
        }
    }
}
