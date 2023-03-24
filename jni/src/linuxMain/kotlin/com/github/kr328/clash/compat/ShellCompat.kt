package com.github.kr328.clash.compat

import com.github.kr328.clash.compat.dbus.*
import com.github.kr328.clash.compat.os.syscall
import kotlinx.cinterop.toKString
import linux.*

actual object ShellCompat {
    private fun getVersion(iface: String): UInt {
        val conn = withDBusError("dbus_bus_get") { error ->
            dbus_bus_get(DBusBusType.DBUS_BUS_SESSION, error)
        }!!

        return conn.getProperty(
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            iface,
            "version"
        ).use {
            it.extract {
                getVariant {
                    getUInt()
                }
            }
        }
    }

    private val fileChooserVersion: UInt
        get() = getVersion("org.freedesktop.portal.FileChooser")

    private val openURIVersion: UInt
        get() = getVersion("org.freedesktop.portal.OpenURI")

    actual val supported: Boolean = runCatching {
        fileChooserVersion >= 1u && openURIVersion >= 1u
    }.getOrDefault(false)

    private val FileDescriptor.descriptor: String
        get() = "x11:${toString(16)}"

    actual fun runLaunchFile(window: FileDescriptor?, file: String) {
        val fd = syscall { open(file, O_RDWR or O_CLOEXEC) }

        try {
            val conn = withDBusError("dbus_bus_get") { error ->
                dbus_bus_get(DBusBusType.DBUS_BUS_SESSION, error)
            }!!

            val reply = conn.call(
                "org.freedesktop.portal.Desktop",
                "/org/freedesktop/portal/desktop",
                "org.freedesktop.portal.OpenURI",
                "OpenFile",
            ) {
                putString(window?.descriptor ?: "")
                putFileDescriptor(fd.toLong())
                putArray(emptyList<Unit>(), "{sv}") {}
            }

            dbus_message_unref(reply)
        } finally {
            close(fd)
        }
    }

    actual fun runPickFile(window: FileDescriptor?, title: String, filters: List<PickerFilter>): String? {
        val conn = withDBusError("dbus_bus_get_private") { error ->
            dbus_bus_get_private(DBusBusType.DBUS_BUS_SESSION, error)
        }!!

        try {
            val responsePath = conn.call(
                "org.freedesktop.portal.Desktop",
                "/org/freedesktop/portal/desktop",
                "org.freedesktop.portal.FileChooser",
                "OpenFile",
            ) {
                putString(window?.descriptor ?: "")
                putString(title)
                putDict(
                    mapOf("filters" to filters),
                    "{sv}",
                    keyBuilder = { putString(it) },
                ) {
                    putVariant("a(sa(us))") {
                        putArray(it, "(sa(us))") { filter ->
                            putStruct {
                                putString(filter.name)
                                putArray(filter.extensions, "(us)") { ext ->
                                    putStruct {
                                        putUInt(0u)
                                        putString("*.$ext")
                                    }
                                }
                            }
                        }
                    }
                }
            }.use { reply ->
                reply.extract {
                    getObjectPath()
                }
            }

            withDBusError("dbus_bus_add_match") { error ->
                dbus_bus_add_match(conn, "type='signal'", error)
            }

            dbus_connection_flush(conn)

            while (dbus_connection_get_is_connected(conn) != 0u) {
                dbus_connection_read_write(conn, DBUS_TIMEOUT_INFINITE)

                dbus_connection_pop_message(conn)?.use { signal ->
                    if (dbus_message_is_signal(signal, "org.freedesktop.portal.Request", "Response") != 0u &&
                        dbus_message_get_path(signal)?.toKString() == responsePath
                    ) {
                        return signal.extract {
                            if (getUInt() == 0u) {
                                val result: Map<String, String?> = getDict(keyExtractor = { getString() }) {
                                    if (it == "uris") {
                                        getVariant {
                                            getArray {
                                                getString()
                                            }
                                        }.firstOrNull()
                                    } else {
                                        null
                                    }
                                }

                                result["uris"]
                            } else {
                                null
                            }
                        }
                    }
                }
            }

            return null
        } catch (e: Exception) {
            e.printStackTrace()

            throw e
        } finally {
            dbus_connection_close(conn)
            dbus_connection_unref(conn)
        }
    }
}
