package com.github.kr328.clash.compat.dbus

import cnames.structs.DBusMessage
import com.github.kr328.clash.compat.FileDescriptor
import com.github.kr328.clash.compat.SyscallException
import kotlinx.cinterop.*
import linux.*

class Builder(val iterator: CPointer<DBusMessageIter>) {
    private inline fun iteratorCall(key: String, block: () -> dbus_bool_t) {
        if (block() == 0u) {
            throw SyscallException("Call dbus iterator $key failed")
        }
    }

    private inline fun openInner(
        key: String,
        type: Int,
        signature: String?,
        block: (CPointer<DBusMessageIter>) -> Unit,
    ): Unit = memScoped {
        val inner: DBusMessageIter = alloc()

        iteratorCall(key) {
            dbus_message_iter_open_container(iterator, type, signature, inner.ptr)
        }

        try {
            block(inner.ptr)
        } finally {
            dbus_message_iter_close_container(iterator, inner.ptr)
        }
    }

    fun putUInt(value: UInt): Unit = memScoped {
        val v: UIntVar = alloc(value)

        iteratorCall("putUInt") {
            dbus_message_iter_append_basic(iterator, DBUS_TYPE_UINT32, v.ptr)
        }
    }

    fun putString(value: String): Unit = memScoped {
        val v: CPointerVar<ByteVar> = alloc<CPointerVar<ByteVar>>().also {
            it.value = value.cstr.ptr
        }

        iteratorCall("putString") {
            dbus_message_iter_append_basic(iterator, DBUS_TYPE_STRING, v.ptr)
        }
    }

    fun putStruct(builder: Builder.() -> Unit) {
        openInner("putStruct", DBUS_TYPE_STRUCT, null) { iterator ->
            Builder(iterator).builder()
        }
    }

    fun putFileDescriptor(fd: FileDescriptor) = memScoped {
        val v: IntVar = alloc<IntVar>().also {
            it.value = fd.toInt()
        }

        iteratorCall("putFileDescriptor") {
            dbus_message_iter_append_basic(iterator, DBUS_TYPE_UNIX_FD, v.ptr)
        }
    }

    fun putVariant(signature: String, builder: Builder.() -> Unit) {
        openInner("putVariant", DBUS_TYPE_VARIANT, signature) { iterator ->
            Builder(iterator).builder()
        }
    }

    fun <T> putArray(value: List<T>, signature: String, builder: Builder.(T) -> Unit) {
        openInner("putArray", DBUS_TYPE_ARRAY, signature) { iterator ->
            value.forEach {
                Builder(iterator).builder(it)
            }
        }
    }

    fun <K, V> putDict(
        map: Map<K, V>,
        signature: String,
        keyBuilder: Builder.(K) -> Unit,
        valueBuilder: Builder.(V) -> Unit
    ) {
        putArray(map.entries.toList(), signature) { kv ->
            openInner("putMapEntry", DBUS_TYPE_DICT_ENTRY, null) { iterator ->
                Builder(iterator).keyBuilder(kv.key)
                Builder(iterator).valueBuilder(kv.value)
            }
        }
    }
}

fun CPointer<DBusMessage>.build(builder: Builder.() -> Unit) = memScoped {
    val iterator: DBusMessageIter = alloc()

    dbus_message_iter_init_append(this@build, iterator.ptr)

    Builder(iterator.ptr).builder()
}
