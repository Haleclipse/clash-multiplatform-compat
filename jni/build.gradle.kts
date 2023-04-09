plugins {
    `maven-publish`
}

task("assembleDebug") {
    group = "build"
}

task("assembleRelease") {
    group = "build"
}

task("assemble") {
    group = "build"

    dependsOn(tasks["assembleDebug"], tasks["assembleRelease"])
}

var targets = listOf(
    Triple("linux-amd64", "x86_64-unknown-linux-gnu", "libcompat.so"),
    Triple("windows-amd64", "x86_64-pc-windows-gnu", "compat.dll"),
)
targets.forEach { (id, target, fileName) ->
    fun configureTask(debug: Boolean) {
        task("compileRust${if (debug) "Debug" else "Release"}[$id]", type = Exec::class) {
            group = "build"

            inputs.dir(file("crate/src"))
            inputs.files(file("crate/Cargo.toml"), file("crate/Cargo.lock"), file("crate/Cross.toml"))
            outputs.file(file("crate/target/$target/${if (debug) "debug" else "release"}/$fileName"))

            commandLine(listOf("cross", "build", "--target", target) + if (!debug) listOf("--release") else emptyList())
            workingDir(file("crate"))
        }
    }

    configureTask(true)
    configureTask(false)
}

task("clean", type = Delete::class) {
    group = "build"

    delete(buildDir)
    delete("crate/target")
}

publishing {
    publications {
        fun configurePublication(id: String, debug: Boolean) {
            val name = id.replace(Regex("-([a-z])")) { it.groups[1]!!.value }.replaceFirstChar { it.uppercase() }
            val buildType = if (debug) "Debug" else "Release"

            create("compat$name$buildType", type = MavenPublication::class) {
                artifactId = if (debug) {
                    "compat-$id-debug"
                } else {
                    "compat-$id"
                }

                val jar = tasks.register("bundleJar$buildType[$id]", type = Jar::class) {
                    destinationDirectory.set(buildDir.resolve("jars"))
                    archiveBaseName.set("$id-${if (debug) "debug" else "release"}")
                    entryCompression = ZipEntryCompression.DEFLATED
                    isPreserveFileTimestamps = false

                    from(tasks["compileRust$buildType[$id]"])
                }

                artifact(jar)
            }
        }

        targets.forEach { (id, _, _) ->
            configurePublication(id, true)
            configurePublication(id, false)
        }
    }
}
