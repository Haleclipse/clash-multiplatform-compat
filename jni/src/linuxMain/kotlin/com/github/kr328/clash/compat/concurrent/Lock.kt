package com.github.kr328.clash.compat.concurrent

import kotlinx.cinterop.alloc
import kotlinx.cinterop.free
import kotlinx.cinterop.nativeHeap
import kotlinx.cinterop.ptr
import linux.*
import kotlin.native.internal.Cleaner
import kotlin.native.internal.createCleaner

class Mutex {
    private val lock: pthread_mutex_t = nativeHeap.alloc()

    init {
        pthread_mutex_init(lock.ptr, null)
    }

    @Suppress("unused")
    @ExperimentalStdlibApi
    private val cleaner: Cleaner = createCleaner(lock) {
        pthread_mutex_destroy(it.ptr)

        nativeHeap.free(it)
    }

    fun lock() {
        pthread_mutex_lock(lock.ptr)
    }

    fun unlock() {
        pthread_mutex_unlock(lock.ptr)
    }
}

inline fun <R> Mutex.withLock(block: () -> R): R {
    lock()

    return try {
        block()
    } finally {
        unlock()
    }
}
