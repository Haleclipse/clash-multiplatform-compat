@file:Suppress("unused", "unused_parameter")

package com.github.kr328.clash.compat

import com.github.kr328.clash.compat.helper.rethrowJavaException
import com.github.kr328.clash.compat.helper.toKString
import com.github.kr328.clash.compat.helper.toList
import kotlinx.cinterop.CPointer
import java.*

expect object ProcessCompat {
    fun createProcess(
        executablePath: String,
        arguments: List<String>,
        workingDir: String,
        environments: List<String>,
        extraFds: List<FileDescriptor>,
        fdStdin: FileDescriptor?,
        fdStdout: FileDescriptor?,
        fdStderr: FileDescriptor?,
    ): FileDescriptor

    fun waitProcess(processFd: FileDescriptor): Int
    fun killProcess(processFd: FileDescriptor)
    fun releaseProcess(processFd: FileDescriptor)
}

@CName("Java_com_github_kr328_clash_compat_ProcessCompat_nativeCreateProcess")
fun nativeCreateProcess(
    env: CPointer<JNIEnvVar>,
    clazz: jclass,
    executablePath: jstring,
    arguments: jobjectArray,
    workingDir: jstring,
    environments: jobjectArray,
    extraFds: jobjectArray,
    fdStdin: jobject?,
    fdStdout: jobject?,
    fdStderr: jobject?,
): jlong {
    return env.rethrowJavaException {
        val kExecutablePath = executablePath.toKString(env)
        val kArguments = arguments.toList(env).map { it!!.toKString(env) }
        val kWorkingDir = workingDir.toKString(env)
        val kEnvironments = environments.toList(env).map { it!!.toKString(env) }
        val kExtraFds = extraFds.toList(env).map { nativeGetFileDescriptorHandle(env, clazz, it!!) }
        val kFdStdin = fdStdin?.let { nativeGetFileDescriptorHandle(env, clazz, it) }
        val kFdStdout = fdStdout?.let { nativeGetFileDescriptorHandle(env, clazz, it) }
        val kFdStderr = fdStderr?.let { nativeGetFileDescriptorHandle(env, clazz, it) }

        ProcessCompat.createProcess(
            kExecutablePath,
            kArguments,
            kWorkingDir,
            kEnvironments,
            kExtraFds,
            kFdStdin,
            kFdStdout,
            kFdStderr,
        )
    } ?: 0L
}

@CName("Java_com_github_kr328_clash_compat_ProcessCompat_nativeWaitProcess")
fun nativeWaitProcess(env: CPointer<JNIEnvVar>, clazz: jclass, handle: jlong): jint {
    return ProcessCompat.waitProcess(handle)
}

@CName("Java_com_github_kr328_clash_compat_ProcessCompat_nativeKillProcess")
fun nativeKillProcess(env: CPointer<JNIEnvVar>, clazz: jclass, handle: jlong) {
    ProcessCompat.killProcess(handle)
}

@CName("Java_com_github_kr328_clash_compat_ProcessCompat_nativeReleaseProcess")
fun nativeReleaseProcess(env: CPointer<JNIEnvVar>, clazz: jclass, handle: jlong) {
    ProcessCompat.releaseProcess(handle)
}
