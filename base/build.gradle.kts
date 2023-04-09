plugins {
    java
    `maven-publish`
}

dependencies {
    implementation("org.jetbrains:annotations:24.0.0")
    testImplementation("org.junit.jupiter:junit-jupiter:5.9.2")
}

java {
    withSourcesJar()
}

tasks.test {
    useJUnitPlatform()

    jvmArgs = jvmArgs!! + listOf(
        "--add-opens", "java.base/sun.nio.ch=ALL-UNNAMED",
        "--add-opens", "java.base/java.io=ALL-UNNAMED",
        "--add-opens", "java.desktop/java.awt=ALL-UNNAMED",
        "--add-opens", "java.desktop/sun.awt.windows=ALL-UNNAMED",
        "--add-opens", "java.desktop/sun.awt.X11=ALL-UNNAMED",
    )
}

publishing {
    publications {
        create("compat", MavenPublication::class) {
            artifactId = "compat"

            from(components["java"])
        }
    }
}
