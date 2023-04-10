plugins {
    `maven-publish`
}

fun configureTarget(id: String, target: String, debug: Boolean, fileName: String, bundleName: String) {
    val compileTask = tasks.register("compileRust[$id]", type = Exec::class) {
        group = "build"

        inputs.dir(file("crate/src"))
        inputs.files(file("crate/Cargo.toml"), file("crate/Cargo.lock"), file("crate/Cross.toml"))
        outputs.file(file("crate/target/$target/${if (debug) "debug" else "release"}/$fileName"))

        commandLine(listOf("cross", "build", "--target", target) + if (!debug) listOf("--release") else emptyList())
        workingDir(file("crate"))
    }

    val jar = tasks.register("bundleJar[$id]", type = Jar::class) {
        destinationDirectory.set(buildDir.resolve("jars"))
        archiveBaseName.set(id)
        entryCompression = ZipEntryCompression.DEFLATED
        isPreserveFileTimestamps = false

        from(compileTask) {
            into("/com/github/kr328/clash/compat/")

            rename { bundleName }
        }
    }

    val publishName = id.replace(Regex("^([a-z])|-([a-z])")) {
        (it.groups[1] ?: it.groups[2])!!.value.replaceFirstChar { c -> c.uppercase() }
    }

    publishing.publications.register("compat$publishName", type = MavenPublication::class) {
        artifactId = "compat-$id"

        artifact(jar)
    }

    configurations.register(id) {
        isCanBeConsumed = true
        isCanBeResolved = false
    }

    artifacts.add(id, jar)
}

configureTarget("linux-amd64", "x86_64-unknown-linux-gnu", false, "libcompat.so", "libcompat-amd64.so")
configureTarget("linux-amd64-debug", "x86_64-unknown-linux-gnu", true, "libcompat.so", "libcompat-amd64.so")
configureTarget("windows-amd64", "x86_64-pc-windows-gnu", false, "compat.dll", "compat-amd64.dll")
configureTarget("windows-amd64-debug", "x86_64-pc-windows-gnu", true, "compat.dll", "compat-amd64.dll")

task("clean", type = Delete::class) {
    group = "build"

    delete(buildDir)
    delete("crate/target")
}
