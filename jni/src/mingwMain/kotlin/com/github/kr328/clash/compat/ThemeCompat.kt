package com.github.kr328.clash.compat

import com.github.kr328.clash.compat.concurrent.thread
import com.github.kr328.clash.compat.os.syscall
import kotlinx.cinterop.*
import windows.*
import kotlin.native.concurrent.AtomicInt

actual object ThemeCompat {
    private val personalizeKey: HKEYVar = nativeHeap.alloc<HKEYVar>().also {
        try {
            syscall("RegOpenKeyExA") {
                RegOpenKeyExA(
                    HKEY_CURRENT_USER,
                    "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
                    0,
                    KEY_READ,
                    it.ptr,
                )
            }
        } catch (e: Exception) {
            it.value = null
        }
    }

    actual val supported: Boolean
        get() = personalizeKey.value != null

    actual fun isNight(): Boolean = memScoped {
        val result: DWORDVar = alloc<DWORDVar>().also {
            it.value = 1u
        }
        val resultLength: DWORDVar = alloc<DWORDVar>().also {
            it.value = sizeOf<DWORDVar>().toUInt()
        }

        syscall("RegQueryValueExA") {
            RegQueryValueExA(
                personalizeKey.value,
                "AppsUseLightTheme",
                null,
                null,
                result.ptr.reinterpret(),
                resultLength.ptr,
            )
        }

        result.value == 0u
    }

    private class Monitor(private val event: HANDLE) : Disposable {
        private val closedInt = AtomicInt(0)

        val closed: Boolean
            get() = closedInt.value != 0

        override fun dispose() {
            closedInt.value = 1

            SetEvent(event)
        }
    }

    actual fun addListener(listener: OnThemeChangedListener): Disposable {
        isNight() // Enforce key available

        val event = syscall("CreateEventA") {
            CreateEventA(null, TRUE, FALSE, null)
        }!!

        val monitor = Monitor(event)

        thread {
            try {
                while (!monitor.closed) {
                    syscall("RegNotifyChangeKeyValue") {
                        RegNotifyChangeKeyValue(
                            personalizeKey.value,
                            FALSE,
                            (REG_NOTIFY_CHANGE_LAST_SET or REG_NOTIFY_CHANGE_NAME).toUInt(),
                            event,
                            TRUE,
                        )
                    }

                    WaitForSingleObject(event, INFINITE)

                    listener.onChanged(isNight())
                }

                listener.onExited()
            } catch (e: Exception) {
                runCatching {
                    listener.onError(e)
                }
            } finally {
                listener.onFinalize()
            }
        }

        return monitor
    }
}
