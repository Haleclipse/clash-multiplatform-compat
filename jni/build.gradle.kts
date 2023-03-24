import org.jetbrains.kotlin.gradle.plugin.mpp.KotlinNativeTarget
import org.jetbrains.kotlin.konan.target.Family
import java.net.URL

plugins {
    kotlin("multiplatform")
    kotlin("plugin.serialization")
    `maven-publish`
}

kotlin {
    targets {
        mingwX64()
        linuxX64()
    }
}

val packagesRoot = buildDir.resolve("kotlin-dependencies")

task("installPackages") {
    val packages: List<Triple<String, String, String>> = listOf(
        Triple("core", "x86_64", "dbus"),
        Triple("extra", "x86_64", "libx11"),
        Triple("extra", "any", "xorgproto"),
    )

    inputs.property("packages", packages)
    outputs.dir(packagesRoot)

    doFirst {
        packages.forEach { (repository, architecture, packageName) ->
            val packageDir = packagesRoot.resolve(packageName).resolve(architecture)
            if (!packageDir.resolve(".PKGINFO").exists()) {
                println("Installing $repository-$architecture-$packageName")

                URL("https://www.archlinux.org/packages/$repository/$architecture/$packageName/download").openStream()
                    .use {
                        packageDir.mkdirs()

                        exec {
                            commandLine(
                                "tar",
                                "xv",
                                "--zstd",
                                "-C",
                                packagesRoot.resolve(packageName).resolve(architecture)
                            )
                            errorOutput = System.out
                            standardOutput = System.out
                            standardInput = it
                        }
                    }
            }
        }
    }
}

kotlin {
    targets {
        withType(KotlinNativeTarget::class) {
            binaries {
                sharedLib {
                    baseName = "compat"
                }
            }
        }
    }

    sourceSets {
        val mingw = create("mingwMain") {
            dependsOn(sourceSets["commonMain"])
        }

        sourceSets["mingwX64Main"].dependsOn(mingw)

        val linux = create("linuxMain") {
            dependsOn(sourceSets["commonMain"])
        }

        sourceSets["linuxX64Main"].dependsOn(linux)
    }

    targets.withType(KotlinNativeTarget::class) {
        compilations["main"].cinterops.create("java") {
            defFile(file("src/commonMain/cinterops/java.def"))
            packageName("java")

            includeDirs(file("src/commonMain/cinterops/include"))

            when (konanTarget.family) {
                Family.MINGW -> {
                    includeDirs(file("src/commonMain/cinterops/include/win32"))
                }
                Family.LINUX -> {
                    includeDirs(file("src/commonMain/cinterops/include/linux"))
                }
                else -> {
                    throw IllegalArgumentException("Unsupported target $konanTarget")
                }
            }

            afterEvaluate {
                tasks[interopProcessingTaskName].dependsOn(tasks["installPackages"])
            }
        }

        when (konanTarget.family) {
            Family.MINGW -> {
                compilations["main"].cinterops.create("windows") {
                    defFile("src/mingwMain/cinterops/windows.def")
                    packageName("windows")
                    extraOpts("-no-default-libs")

                    afterEvaluate {
                        tasks[interopProcessingTaskName].dependsOn(tasks["installPackages"])
                    }
                }
            }
            Family.LINUX -> {
                compilations["main"].cinterops.create("linux") {
                    defFile("src/linuxMain/cinterops/linux.def")
                    packageName("linux")
                    extraOpts("-no-default-libs")

                    includeDirs(
                        packagesRoot.resolve("dbus/x86_64/usr/include/dbus-1.0"),
                        packagesRoot.resolve("dbus/x86_64/usr/lib/dbus-1.0/include"),
                        packagesRoot.resolve("libx11/x86_64/usr/include"),
                        packagesRoot.resolve("xorgproto/any/usr/include"),
                    )

                    binaries.all {
                        linkerOpts += "-L" + packagesRoot.resolve("dbus/x86_64/usr/lib")
                        linkerOpts += "-L" + packagesRoot.resolve("libx11/x86_64/usr/lib")
                    }

                    afterEvaluate {
                        tasks[interopProcessingTaskName].dependsOn(tasks["installPackages"])
                    }
                }
            }
            else -> Unit
        }
    }
}

publishing {
    publications {
        val variants: List<Triple<String, String, String>> = listOf(
            Triple("linux-amd64", "linkReleaseSharedLinuxX64", "libcompat-amd64.so"),
            Triple("linux-amd64-debug", "linkDebugSharedLinuxX64", "libcompat-amd64.so"),
            Triple("windows-amd64", "linkReleaseSharedMingwX64", "libcompat-amd64.dll"),
            Triple("windows-amd64-debug", "linkDebugSharedMingwX64", "libcompat-amd64.dll"),
        )

        variants.forEach { (id: String, taskId: String, fileName: String) ->
            val publishName = id.replace(Regex("-[a-z]")) {
                it.value[1].uppercase()
            }

            create("compat${publishName.replaceFirstChar { it.uppercase() }}", MavenPublication::class) {
                artifactId = "compat-$id"

                val jarTask = tasks.register(
                    "jniLibsJar[$id]",
                    type = Jar::class
                ) {
                    archiveBaseName.set(id)

                    isPreserveFileTimestamps = false
                    entryCompression = ZipEntryCompression.DEFLATED

                    from(tasks[taskId])
                    into("com/github/kr328/clash/compat/")

                    include("*.${fileName.split(".").last()}")

                    rename { fileName }
                }

                artifact(jarTask)
            }
        }
    }
}
