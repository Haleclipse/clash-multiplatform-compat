package com.github.kr328.clash.compat

import com.github.kr328.clash.compat.os.syscall
import kotlinx.cinterop.*
import linux.*

actual object FileCompat {
    actual val placements: FileDescriptorPlacement = FileDescriptorPlacement(
        file = FileDescriptorPlacement.Placement.Fd,
        socket = FileDescriptorPlacement.Placement.Fd,
    )

    actual fun closeFileDescriptor(fd: FileDescriptor) {
        close(fd.toInt())
    }

    actual fun setFileDescriptorInheritable(fd: FileDescriptor, inheritable: Boolean) {
        val flags = syscall { fcntl(fd.toInt(), F_GETFD) }

        if (inheritable) {
            syscall { fcntl(fd.toInt(), F_SETFD, flags and FD_CLOEXEC.inv()) }
        } else {
            syscall { fcntl(fd.toInt(), F_SETFD, flags or FD_CLOEXEC) }
        }
    }

    actual fun createSocketPair(): Pair<FileDescriptor, FileDescriptor> = memScoped {
        val pair: CArrayPointer<IntVar> = allocArray(2)

        syscall { socketpair(AF_UNIX, SOCK_STREAM, 0, pair) }

        setFileDescriptorInheritable(pair[0].toLong(), false)
        setFileDescriptorInheritable(pair[1].toLong(), false)

        pair[0].toLong() to pair[1].toLong()
    }

    actual fun createPipe(): Pair<FileDescriptor, FileDescriptor> = memScoped {
        val pipe: CArrayPointer<IntVar> = allocArray(2)

        try {
            syscall { pipe(pipe) }

            setFileDescriptorInheritable(pipe[0].toLong(), false)
            setFileDescriptorInheritable(pipe[1].toLong(), false)

            val reader = pipe[0].toLong()
            val writer = pipe[1].toLong()

            pipe[0] = -1
            pipe[1] = -1

            reader to writer
        } finally {
            closeFileDescriptor(pipe[0].toLong())
            closeFileDescriptor(pipe[1].toLong())
        }
    }
}
