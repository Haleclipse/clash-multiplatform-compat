package com.github.kr328.clash.compat.dbus

import kotlinx.cinterop.CPointer
import linux.DBusMessage
import linux.dbus_message_unref

inline fun <R>CPointer<DBusMessage>.use(block: (CPointer<DBusMessage>) -> R): R {
    return try {
        block(this)
    } finally {
        dbus_message_unref(this)
    }
}
