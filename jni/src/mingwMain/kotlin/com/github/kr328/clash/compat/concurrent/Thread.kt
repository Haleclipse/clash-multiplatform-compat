package com.github.kr328.clash.compat.concurrent

import com.github.kr328.clash.compat.os.syscall
import kotlinx.cinterop.*
import windows.CreateThread
import windows.DWORD
import windows.LPVOID

fun thread(block: () -> Unit) {
    val routine: CPointer<CFunction<(LPVOID?) -> DWORD>> = staticCFunction { arg ->
        val ref = arg!!.asStableRef<() -> Unit>()

        val blockRef = ref.get()

        ref.dispose()

        blockRef()

        0u
    }

    val blockRef = StableRef.create(block)

    try {
        syscall("CreateThread") {
            CreateThread(
                null,
                1024u,
                routine,
                blockRef.asCPointer(),
                0u,
                null,
            )
        }
    } catch (e: Exception) {
        blockRef.dispose()

        throw e
    }
}
