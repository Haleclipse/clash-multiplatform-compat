@file:Suppress("unused", "unused_parameter")

package com.github.kr328.clash.compat

import com.github.kr328.clash.compat.helper.*
import kotlinx.cinterop.*
import java.*

typealias FileDescriptor = Long

data class FileDescriptorPlacement(val file: Placement, val socket: Placement) {
    enum class Placement {
        Fd, Handle,
    }
}

expect object FileCompat {
    val placements: FileDescriptorPlacement

    fun closeFileDescriptor(fd: FileDescriptor)
    fun setFileDescriptorInheritable(fd: FileDescriptor, inheritable: Boolean)
    fun createSocketPair(): Pair<FileDescriptor, FileDescriptor>
    fun createPipe(): Pair<FileDescriptor, FileDescriptor>
}

private val fSocketChannelFd = LazyField(
    cfg = LazyMemberConfig(
        className = "sun/nio/ch/SocketChannelImpl",
        name = "fd",
        signature = "Ljava/io/FileDescriptor;",
        isStatic = false,
    ),
)

@CName("Java_com_github_kr328_clash_compat_FileCompat_nativeGetFileDescriptorFromSocketChannel")
fun nativeGetFileDescriptorFromSocketChannel(env: CPointer<JNIEnvVar>, clazz: jclass, channel: jobject): jobject {
    return env.functions {
        GetObjectField!!(env, channel, fSocketChannelFd.get(env))!!
    }
}

private val fFileDescriptorFd = LazyField(
    cfg = LazyMemberConfig(
        className = "java/io/FileDescriptor",
        name = "fd",
        signature = "I",
        isStatic = false,
    ),
)

private val fFileDescriptorHandle = LazyField(
    cfg = LazyMemberConfig(
        className = "java/io/FileDescriptor",
        name = "handle",
        signature = "J",
        isStatic = false,
    ),
)

@CName("Java_com_github_kr328_clash_compat_FileCompat_nativeGetFileDescriptorHandle")
fun nativeGetFileDescriptorHandle(env: CPointer<JNIEnvVar>, clazz: jclass, fd: jobject): jlong {
    return env.functions {
        val handle = GetLongField!!(env, fd, fFileDescriptorHandle.get(env))
        if (handle >= 0) {
            handle
        } else {
            GetIntField!!(env, fd, fFileDescriptorFd.get(env)).toLong()
        }
    }
}

@CName("Java_com_github_kr328_clash_compat_FileCompat_nativeSetFileDescriptorInheritable")
fun setFileDescriptorInheritable(env: CPointer<JNIEnvVar>, clazz: jclass, fd: jobject, inheritable: jboolean) {
    env.rethrowJavaException {
        val handle = env.rethrowKotlinException {
            nativeGetFileDescriptorHandle(env, clazz, fd)
        }

        FileCompat.setFileDescriptorInheritable(handle, inheritable.asBoolean())
    }
}

@CName("Java_com_github_kr328_clash_compat_FileCompat_nativeCloseFileDescriptor")
fun nativeCloseFileDescriptor(env: CPointer<JNIEnvVar>, clazz: jclass, fd: jobject) {
    env.rethrowJavaException {
        val handle = env.rethrowKotlinException {
            nativeGetFileDescriptorHandle(env, clazz, fd)
        }

        FileCompat.closeFileDescriptor(handle)
    }
}

private fun setFileDescriptor(env: CPointer<JNIEnvVar>, fd: jobject, value: Long, socket: Boolean) {
    env.functions {
        val placement = if (socket) {
            FileCompat.placements.socket
        } else {
            FileCompat.placements.file
        }

        when (placement) {
            FileDescriptorPlacement.Placement.Fd -> {
                SetIntField!!(env, fd, fFileDescriptorFd.get(env), value.toInt())
            }
            FileDescriptorPlacement.Placement.Handle -> {
                SetLongField!!(env, fd, fFileDescriptorHandle.get(env), value)
            }
        }
    }
}

@CName("Java_com_github_kr328_clash_compat_FileCompat_nativeCreatePipe")
fun nativeCreatePipe(env: CPointer<JNIEnvVar>, clazz: jclass, reader: jobject, writer: jobject) {
    env.rethrowJavaException {
        env.functions {
            val (readerFd, writerFd) = FileCompat.createPipe()

            setFileDescriptor(env, reader, readerFd, false)
            setFileDescriptor(env, writer, writerFd, false)
        }
    }
}

@CName("Java_com_github_kr328_clash_compat_FileCompat_nativeCreateUnixSocketPair")
fun nativeCreateUnixSocketPair(env: CPointer<JNIEnvVar>, clazz: jclass, first: jobject, second: jobject) {
    env.rethrowJavaException {
        val (firstFd, secondFd) = FileCompat.createSocketPair()

        setFileDescriptor(env, first, firstFd, true)
        setFileDescriptor(env, second, secondFd, true)
    }
}

private val cSocketChannelImpl = LazyClass(className = "sun/nio/ch/SocketChannelImpl")

private val mSocketChannelNew = LazyMethod(
    cfg = LazyMemberConfig(
        className = "sun/nio/ch/SocketChannelImpl",
        name = "<init>",
        signature = "(Ljava/nio/channels/spi/SelectorProvider;Ljava/net/ProtocolFamily;Ljava/io/FileDescriptor;Ljava/net/SocketAddress;)V",
        isStatic = false,
    ),
)

@CName("Java_com_github_kr328_clash_compat_FileCompat_nativeNewSocketChannel")
fun nativeNewSocketChannel(
    env: CPointer<JNIEnvVar>,
    clazz: jclass,
    sp: jobject,
    family: jobject,
    fd: jobject,
    address: jobject
): jobject? {
    return env.functions {
        memScoped {
            val args: CArrayPointer<jvalue> = jValues(4) {
                it[0].l = sp
                it[1].l = family
                it[2].l = fd
                it[3].l = address
            }

            NewObjectA!!(env, cSocketChannelImpl.get(env), mSocketChannelNew.get(env), args)
        }
    }
}
