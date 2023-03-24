package com.github.kr328.clash.compat.os

import com.github.kr328.clash.compat.SyscallException
import kotlinx.cinterop.toKString
import linux.errno
import linux.set_errno
import linux.strerror

fun <R> syscall(block: () -> R): R {
    val lastErrno = errno

    set_errno(0)

    return try {
        block().also {
            if (errno != 0) {
                throw SyscallException(strerror(errno)?.toKString() ?: "Unknown")
            }
        }
    } finally {
        set_errno(lastErrno)
    }
}
