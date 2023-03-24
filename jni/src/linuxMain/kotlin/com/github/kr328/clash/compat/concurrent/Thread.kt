package com.github.kr328.clash.compat.concurrent

import com.github.kr328.clash.compat.SyscallException
import kotlinx.cinterop.*
import linux.pthread_create
import linux.pthread_tVar

private fun threadCallback(arg: COpaquePointer?): COpaquePointer? {
    val ref = arg!!.asStableRef<() -> Unit>()
    val block = ref.get()
    ref.dispose()

    block()

    return null
}

fun thread(block: () -> Unit): Unit = memScoped {
    val tid: pthread_tVar = alloc()
    val ref = StableRef.create(block)

    if (pthread_create(tid.ptr, null, staticCFunction(::threadCallback), ref.asCPointer()) != 0) {
        ref.dispose()

        throw SyscallException("pthread_create")
    }
}
