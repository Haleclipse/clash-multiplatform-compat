package com.github.kr328.clash.compat.os

import com.github.kr328.clash.compat.SyscallException
import kotlinx.cinterop.*
import windows.*

private fun formatError(errno: DWORD): String = memScoped {
    val msg: CPointerVar<ByteVar> = alloc()

    val result = FormatMessageA(
        (FORMAT_MESSAGE_ALLOCATE_BUFFER or FORMAT_MESSAGE_FROM_SYSTEM or FORMAT_MESSAGE_IGNORE_INSERTS).toUInt(),
        null,
        errno,
        makeLanguageId(LANG_ENGLISH, SUBLANG_DEFAULT),
        msg.ptr.reinterpret(),
        0,
        null
    )

    if (result != 0u) {
        msg.value!!.toKString()
    } else {
        "Unknown"
    }
}

fun <R> syscall(name: String, block: () -> R): R {
    val lastError = GetLastError()

    SetLastError(ERROR_SUCCESS)

    return try {
        block().also {
            val errno = GetLastError()
            if (errno != ERROR_SUCCESS.toUInt()) {
                throw SyscallException("$name: ${formatError(errno)}")
            }
        }
    } finally {
        SetLastError(lastError)
    }
}
