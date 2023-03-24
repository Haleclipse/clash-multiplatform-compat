package com.github.kr328.clash.compat

import com.github.kr328.clash.compat.os.syscall
import kotlinx.cinterop.*
import linux.*

actual object ProcessCompat {
    actual fun createProcess(
        executablePath: String,
        arguments: List<String>,
        workingDir: String,
        environments: List<String>,
        extraFds: List<FileDescriptor>,
        fdStdin: FileDescriptor?,
        fdStdout: FileDescriptor?,
        fdStderr: FileDescriptor?,
    ): FileDescriptor = memScoped {
        val executableFd: Int = syscall {
            open(executablePath, O_RDONLY or O_CLOEXEC)
        }
        val argumentsPointers: CArrayPointer<CPointerVar<ByteVar>> = allocArray(arguments.size + 1) {
            value = arguments.getOrNull(it)?.cstr?.ptr
        }
        val workingDirFd = syscall {
            open(workingDir, O_RDONLY or O_CLOEXEC or O_DIRECTORY)
        }
        val environmentsPointers: CArrayPointer<CPointerVar<ByteVar>> = allocArray(environments.size + 1) {
            value = environments.getOrNull(it)?.cstr?.ptr
        }
        val extraFdsPointers: CArrayPointer<IntVar> = allocArray(extraFds.size + 1) {
            value = extraFds.getOrNull(it)?.toInt() ?: -1
        }
        val nullFd: Int = syscall { open("/dev/null", O_RDWR) }

        try {
            fork_exec(
                executableFd,
                argumentsPointers,
                environmentsPointers,
                workingDirFd,
                extraFdsPointers,
                fdStdin?.toInt() ?: nullFd,
                fdStdout?.toInt() ?: nullFd,
                fdStderr?.toInt() ?: nullFd,
            ).toLong()
        } finally {
            close(nullFd)
        }
    }

    actual fun waitProcess(processFd: FileDescriptor): Int = memScoped {
        val ret: IntVar = alloc<IntVar>().also {
            it.value = 1
        }

        waitpid(processFd.toInt(), ret.ptr, 0)

        ret.value
    }

    actual fun killProcess(processFd: FileDescriptor) {
        kill(processFd.toInt(), SIGKILL)
    }

    actual fun releaseProcess(processFd: FileDescriptor) {
        // Do nothing
    }
}
