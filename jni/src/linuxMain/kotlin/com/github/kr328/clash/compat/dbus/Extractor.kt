package com.github.kr328.clash.compat.dbus

import cnames.structs.DBusMessage
import com.github.kr328.clash.compat.SyscallException
import kotlinx.cinterop.*
import linux.*

class Extractor(private val iterator: CPointer<DBusMessageIter>) {
    private inline fun <R> readValue(block: () -> R): R {
        val ret = block()

        dbus_message_iter_next(iterator)

        return ret
    }

    private inline fun enforceType(type: Int) {
        val argType = dbus_message_iter_get_arg_type(iterator)

        if (argType != type) {
            throw SyscallException("Excepted type ${type.toChar()} but got ${argType.toChar()}")
        }
    }

    private inline fun <R> recurseInner(block: (CPointer<DBusMessageIter>) -> R) = memScoped {
        val inner: DBusMessageIter = alloc()

        dbus_message_iter_recurse(iterator, inner.ptr)

        block(inner.ptr)
    }

    fun getUInt(): UInt {
        enforceType(DBUS_TYPE_UINT32)

        return memScoped {
            readValue {
                val value: UIntVar = alloc()

                dbus_message_iter_get_basic(iterator, value.ptr)

                value.value
            }
        }
    }

    fun getString(): String {
        enforceType(DBUS_TYPE_STRING)

        return memScoped {
            readValue {
                val value: CPointerVar<ByteVar> = alloc()

                dbus_message_iter_get_basic(iterator, value.ptr)

                value.value!!.toKString()
            }
        }
    }

    fun getObjectPath(): String {
        enforceType(DBUS_TYPE_OBJECT_PATH)

        return memScoped {
            readValue {
                val value: CPointerVar<ByteVar> = alloc()

                dbus_message_iter_get_basic(iterator, value.ptr)

                value.value!!.toKString()
            }
        }
    }

    fun <T> getVariant(extractor: Extractor.() -> T): T {
        enforceType(DBUS_TYPE_VARIANT)

        return readValue {
            recurseInner {
                Extractor(it).extractor()
            }
        }
    }

    fun <T> getArray(extractor: Extractor.(Int) -> T): List<T> {
        enforceType(DBUS_TYPE_ARRAY)

        val length = dbus_message_iter_get_element_count(iterator)

        return readValue {
            recurseInner { iterator ->
                List(length) {
                    Extractor(iterator).extractor(it)
                }
            }
        }
    }

    fun <K, V> getDict(
        keyExtractor: Extractor.(Int) -> K,
        valueExtractor: Extractor.(K) -> V
    ): Map<K, V> {
        return getArray { idx ->
            enforceType(DBUS_TYPE_DICT_ENTRY)

            readValue {
                recurseInner { iterator ->
                    val k = Extractor(iterator).keyExtractor(idx)
                    val v = Extractor(iterator).valueExtractor(k)

                    k to v
                }
            }
        }.toMap()
    }
}

fun <R> CPointer<DBusMessage>.extract(extractor: Extractor.() -> R): R = memScoped {
    val iterator: DBusMessageIter = alloc()

    if (dbus_message_iter_init(this@extract, iterator.ptr) == 0u) {
        throw SyscallException("Unable to open iterator")
    }

    Extractor(iterator.ptr).extractor()
}
