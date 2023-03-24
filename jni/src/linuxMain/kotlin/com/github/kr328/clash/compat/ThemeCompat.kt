package com.github.kr328.clash.compat

import cnames.structs.DBusConnection
import com.github.kr328.clash.compat.concurrent.thread
import com.github.kr328.clash.compat.dbus.call
import com.github.kr328.clash.compat.dbus.extract
import com.github.kr328.clash.compat.dbus.use
import com.github.kr328.clash.compat.dbus.withDBusError
import kotlinx.cinterop.CPointer
import linux.*

actual object ThemeCompat {
    actual val supported: Boolean = runCatching { isNight() }.isSuccess

    actual fun isNight(): Boolean {
        val conn: CPointer<DBusConnection> = withDBusError("dbus_bus_get") { error ->
            dbus_bus_get(DBusBusType.DBUS_BUS_SESSION, error)
        }!!

        return conn.call(
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.Settings",
            "Read",
        ) {
            putString("org.freedesktop.appearance")
            putString("color-scheme")
        }.use { reply ->
            reply.extract {
                getVariant {
                    getVariant {
                        getUInt()
                    }
                }
            } == 1u
        }
    }

    actual fun addListener(listener: OnThemeChangedListener): Disposable {
        var conn: CPointer<DBusConnection>? = withDBusError("dbus_bus_get") { error ->
            dbus_bus_get_private(DBusBusType.DBUS_BUS_SESSION, error)
        }!!

        withDBusError("dbus_bus_add_match") { error ->
            dbus_bus_add_match(conn, "type='signal',interface='org.freedesktop.portal.Settings'", error)
        }

        val disposable = Disposable {
            conn?.also {
                dbus_connection_close(it)
            }
        }

        thread {
            try {
                while (dbus_connection_get_is_connected(conn) != 0u) {
                    dbus_connection_read_write(conn, DBUS_TIMEOUT_INFINITE)

                    dbus_connection_pop_message(conn)?.use { signal ->
                        if (dbus_message_is_signal(
                                signal,
                                "org.freedesktop.portal.Settings",
                                "SettingChanged",
                            ) != 0u
                        ) {
                            signal.extract {
                                val ns = getString()
                                val name = getString()

                                if (ns == "org.freedesktop.appearance" && name == "color-scheme") {
                                    val value = getVariant {
                                        getUInt()
                                    }

                                    listener.onChanged(value == 1u)
                                }
                            }
                        }
                    }
                }

                listener.onExited()
            } catch (e: Exception) {
                dbus_connection_close(conn)

                listener.onError(e)
            } finally {
                listener.onFinalize()

                conn = null

                dbus_connection_unref(conn)
            }
        }

        return disposable
    }
}
