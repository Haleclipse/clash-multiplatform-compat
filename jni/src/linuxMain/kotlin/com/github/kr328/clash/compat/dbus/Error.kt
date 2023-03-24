package com.github.kr328.clash.compat.dbus

import com.github.kr328.clash.compat.SyscallException
import kotlinx.cinterop.*
import linux.DBusError
import linux.dbus_error_free
import linux.dbus_error_init
import linux.dbus_error_is_set

inline fun <R> withDBusError(key: String, block: (CPointer<DBusError>) -> R): R = memScoped {
    val error: DBusError = alloc()

    dbus_error_init(error.ptr)

    try {
        val ret = block(error.ptr)

        if (dbus_error_is_set(error.ptr) != 0u) {
            throw SyscallException("$key: ${error.message?.toKString()}")
        }

        ret
    } finally {
        dbus_error_free(error.ptr)
    }
}
