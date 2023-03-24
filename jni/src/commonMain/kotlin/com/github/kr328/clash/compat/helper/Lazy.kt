package com.github.kr328.clash.compat.helper

import kotlinx.cinterop.CPointer
import kotlinx.cinterop.cstr
import kotlinx.cinterop.invoke
import kotlinx.cinterop.memScoped
import java.*
import kotlin.native.concurrent.AtomicReference

private sealed class LazyState<C, V> {
    data class Uninitialized<C, T>(val config: C) : LazyState<C, T>()
    data class Initialized<C, T>(val value: T) : LazyState<C, T>()
}

abstract class AbstractLazy<C, V>(val config: C) {
    private val state: AtomicReference<LazyState<C, V>> = AtomicReference(LazyState.Uninitialized(config))

    protected abstract fun create(env: CPointer<JNIEnvVar>, c: C): V
    protected abstract fun drop(env: CPointer<JNIEnvVar>, v: V)

    fun get(env: CPointer<JNIEnvVar>): V {
        val s = state.value
        if (s is LazyState.Initialized) {
            return s.value
        }

        s as LazyState.Uninitialized

        return env.suppressException {
            val v = create(env, s.config)
            if (!state.compareAndSet(s, LazyState.Initialized(v))) {
                drop(env, v)
            }

            (state.value as LazyState.Initialized).value
        }
    }
}

data class LazyMemberConfig(val className: String, val name: String, val signature: String, val isStatic: Boolean)

class LazyMethod(cfg: LazyMemberConfig) : AbstractLazy<LazyMemberConfig, jmethodID>(cfg) {
    override fun create(env: CPointer<JNIEnvVar>, c: LazyMemberConfig): jmethodID {
        return memScoped {
            env.functions {
                val clazz = FindClass!!(env, c.className.cstr.ptr)!!

                val method = if (c.isStatic) {
                    GetStaticMethodID!!(env, clazz, c.name.cstr.ptr, c.signature.cstr.ptr)
                } else {
                    GetMethodID!!(env, clazz, c.name.cstr.ptr, c.signature.cstr.ptr)
                }

                method!!
            }
        }
    }

    override fun drop(env: CPointer<JNIEnvVar>, v: jmethodID) {
        // do nothing
    }
}

class LazyField(cfg: LazyMemberConfig) : AbstractLazy<LazyMemberConfig, jfieldID>(cfg) {
    override fun create(env: CPointer<JNIEnvVar>, c: LazyMemberConfig): jfieldID {
        return memScoped {
            env.functions { env ->
                val clazz = FindClass!!(env, c.className.cstr.ptr)!!

                val field = if (c.isStatic) {
                    GetStaticFieldID!!(env, clazz, c.name.cstr.ptr, c.signature.cstr.ptr)
                } else {
                    GetFieldID!!(env, clazz, c.name.cstr.ptr, c.signature.cstr.ptr)
                }

                field!!
            }
        }
    }

    override fun drop(env: CPointer<JNIEnvVar>, v: jfieldID) {
        // do nothing
    }
}

class LazyClass(className: String) : AbstractLazy<String, jclass>(className) {
    override fun create(env: CPointer<JNIEnvVar>, c: String): jclass {
        return memScoped {
            env.functions {
                val clazz = FindClass!!(env, c.cstr.ptr)!!

                NewGlobalRef!!(env, clazz)!!
            }
        }
    }

    override fun drop(env: CPointer<JNIEnvVar>, v: jclass) {
        env.functions {
            DeleteGlobalRef!!(env, v)
        }
    }
}

class LazyStaticFieldObject(cfg: LazyField) : AbstractLazy<LazyField, jobject>(cfg) {
    override fun create(env: CPointer<JNIEnvVar>, c: LazyField): jobject {
        return memScoped {
            env.functions {
                val clazz = FindClass!!(env, c.config.className.cstr.ptr)!!
                val obj = GetStaticObjectField!!(env, clazz, c.get(env))!!

                NewGlobalRef!!(env, obj)!!
            }
        }
    }

    override fun drop(env: CPointer<JNIEnvVar>, v: jobject) {
        env.functions {
            DeleteGlobalRef!!(env, v)
        }
    }
}
