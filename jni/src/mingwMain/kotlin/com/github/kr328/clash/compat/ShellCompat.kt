package com.github.kr328.clash.compat

import com.github.kr328.clash.compat.os.syscall
import kotlinx.cinterop.*
import windows.*

actual object ShellCompat {
    actual const val supported: Boolean = true

    actual fun runLaunchFile(window: FileDescriptor?, file: String) {
        syscall("ShellExecuteA") {
            ShellExecuteA(
                window?.toCPointer(),
                "open",
                file,
                null,
                null,
                SW_SHOW,
            )
        }
    }

    actual fun runPickFile(window: FileDescriptor?, title: String, filters: List<PickerFilter>): String? = memScoped {
        val joinedFilters: ByteArray = filters.joinToString(separator = "\n", postfix = "\n") {
            it.name + "\n" + it.extensions.joinToString(";")
        }.encodeToByteArray().also {
            for (idx in it.indices) {
                if (it[idx] == '\n'.code.toByte()) {
                    it[idx] = 0
                }
            }
        }

        val path: CArrayPointer<ByteVar> = allocArray(MAX_PATH)

        val ret = joinedFilters.usePinned { pinnedFilters ->
            val openFileName: CValue<OPENFILENAMEA> = cValue {
                lStructSize = sizeOf<OPENFILENAMEA>().toUInt()
                hwndOwner = window?.toCPointer()
                lpstrTitle = title.cstr.ptr
                lpstrFilter = pinnedFilters.addressOf(0)
                lpstrFile = path.pointed.ptr
                nMaxFile = (MAX_PATH - 1).toUInt()
                lpstrInitialDir = getenv("USERPROFILE")
                Flags = (OFN_PATHMUSTEXIST or OFN_FILEMUSTEXIST).toUInt()
            }

            GetOpenFileNameA(openFileName.ptr)
        }

        if (ret == TRUE) {
            path.toKString()
        } else {
            null
        }
    }
}
