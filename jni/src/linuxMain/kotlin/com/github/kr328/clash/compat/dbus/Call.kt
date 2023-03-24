package com.github.kr328.clash.compat.dbus

import cnames.structs.DBusConnection
import cnames.structs.DBusMessage
import kotlinx.cinterop.CPointer
import kotlinx.cinterop.memScoped
import linux.DBUS_TIMEOUT_INFINITE
import linux.dbus_connection_send_with_reply_and_block
import linux.dbus_message_new_method_call
import linux.dbus_message_unref

fun CPointer<DBusConnection>.call(
    busName: String,
    path: String,
    iface: String,
    method: String,
    builder: Builder.() -> Unit,
): CPointer<DBusMessage> = memScoped {
    val message = dbus_message_new_method_call(busName, path, iface, method)!!

    try {
        message.build(builder)

        withDBusError("dbus_connection_send_with_reply_and_block") { error ->
            dbus_connection_send_with_reply_and_block(
                this@call,
                message,
                DBUS_TIMEOUT_INFINITE,
                error,
            )
        }!!
    } finally {
        dbus_message_unref(message)
    }
}

fun CPointer<DBusConnection>.getProperty(
    busName: String,
    path: String,
    iface: String,
    property: String,
): CPointer<DBusMessage> {
    return call(
        busName,
        path,
        "org.freedesktop.DBus.Properties",
        "Get",
    ) {
        putString(iface)
        putString(property)
    }
}
