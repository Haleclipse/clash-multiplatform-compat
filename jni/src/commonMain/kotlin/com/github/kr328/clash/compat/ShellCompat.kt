@file:Suppress("unused", "unused_parameter")

package com.github.kr328.clash.compat

import com.github.kr328.clash.compat.helper.*
import kotlinx.cinterop.CPointer
import kotlinx.cinterop.invoke
import kotlinx.cinterop.memScoped
import java.*

data class PickerFilter(val name: String, val extensions: List<String>)

expect object ShellCompat {
    val supported: Boolean

    fun runLaunchFile(window: FileDescriptor?, file: String)
    fun runPickFile(window: FileDescriptor?, title: String, filters: List<PickerFilter>): String?
}

@CName("Java_com_github_kr328_clash_compat_ShellCompat_nativeIsSupported")
fun nativeShellCompatSupported(env: CPointer<JNIEnvVar>, clazz: jclass): jboolean {
    return (if (ShellCompat.supported) JNI_TRUE else JNI_FALSE).toUByte()
}

@CName("Java_com_github_kr328_clash_compat_ShellCompat_nativeRunLaunchFile")
fun nativeRunLaunchFile(env: CPointer<JNIEnvVar>, clazz: jclass, window: jlong, file: jstring) {
    env.rethrowJavaException {
        ShellCompat.runLaunchFile(window.takeIf { it != 0L }, file.toKString(env))
    }
}

private val fPickerFilterName = LazyMethod(
    cfg = LazyMemberConfig(
        className = "com/github/kr328/clash/compat/ShellCompat\$NativePickerFilter",
        name = "name",
        signature = "()Ljava/lang/String;",
        isStatic = false,
    ),
)

private val fPickerFilterExtensions = LazyMethod(
    cfg = LazyMemberConfig(
        className = "com/github/kr328/clash/compat/ShellCompat\$NativePickerFilter",
        name = "extensions",
        signature = "()[Ljava/lang/String;",
        isStatic = false,
    ),
)

@CName("Java_com_github_kr328_clash_compat_ShellCompat_nativeRunPickFile")
fun nativeRunPickFile(env: CPointer<JNIEnvVar>, clazz: jclass, window: jlong, title: jstring, filters: jobjectArray): jstring? {
    return env.rethrowJavaException {
        val kFilters = env.functions {
            memScoped {
                filters.toList(env).map {
                    val name = CallObjectMethodA!!(
                        env,
                        it!!,
                        fPickerFilterName.get(env),
                        jValues(0),
                    )!!.toKString(env)

                    val extensions = CallObjectMethodA!!(
                        env,
                        it,
                        fPickerFilterExtensions.get(env),
                        jValues(0),
                    )!!.toList(env).map { s ->
                        s!!.toKString(env)
                    }

                    PickerFilter(name, extensions)
                }
            }
        }

        ShellCompat.runPickFile(window.takeIf { it != 0L }, title.toKString(env), kFilters)?.toJString(env)
    }
}
