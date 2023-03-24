@file:Suppress("unused", "unused_parameter")

package com.github.kr328.clash.compat

import com.github.kr328.clash.compat.helper.*
import kotlinx.cinterop.*
import java.*

interface OnThemeChangedListener {
    fun onChanged(value: Boolean)
    fun onExited()
    fun onError(e: Exception)
    fun onFinalize()
}

expect object ThemeCompat {
    val supported: Boolean

    fun isNight(): Boolean
    fun addListener(listener: OnThemeChangedListener): Disposable
}

@CName("Java_com_github_kr328_clash_compat_ThemeCompat_nativeIsSupported")
fun nativeIsSupported(env: CPointer<JNIEnvVar>, clazz: jclass): jboolean {
    return (if (ThemeCompat.supported) JNI_TRUE else JNI_FALSE).toUByte()
}

@CName("Java_com_github_kr328_clash_compat_ThemeCompat_nativeIsNight")
fun nativeIsNight(env: CPointer<JNIEnvVar>, clazz: jclass): jboolean {
    return env.rethrowJavaException {
        (if (ThemeCompat.isNight()) JNI_TRUE else JNI_FALSE).toUByte()
    } ?: JNI_FALSE.toUByte()
}

private val mOnThemeChangedListenerOnChanged = LazyMethod(
    cfg = LazyMemberConfig(
        className = "com/github/kr328/clash/compat/ThemeCompat\$OnThemeChangedListener",
        name = "onChanged",
        signature = "(Z)V",
        isStatic = false,
    ),
)

private val mOnThemeChangedListenerOnExited = LazyMethod(
    cfg = LazyMemberConfig(
        className = "com/github/kr328/clash/compat/ThemeCompat\$OnThemeChangedListener",
        name = "onExited",
        signature = "()V",
        isStatic = false,
    ),
)

private val mOnThemeChangedListenerOnError = LazyMethod(
    cfg = LazyMemberConfig(
        className = "com/github/kr328/clash/compat/ThemeCompat\$OnThemeChangedListener",
        name = "onError",
        signature = "(Ljava/lang/Exception;)V",
        isStatic = false,
    ),
)

@CName("Java_com_github_kr328_clash_compat_ThemeCompat_nativeAddListener")
fun nativeAddListener(env: CPointer<JNIEnvVar>, clazz: jclass, listener: jobject): jlong {
    return env.rethrowJavaException {
        env.functions {
            val gListener = NewGlobalRef!!(env, listener)!!

            val kListener = object : OnThemeChangedListener {
                override fun onChanged(value: Boolean) {
                    globalJavaVM.withAttachedThread { env ->
                        env.functions {
                            memScoped {
                                val args: CArrayPointer<jvalue> = jValues(1) {
                                    it[0].z = (if (value) JNI_TRUE else JNI_FALSE).toUByte()
                                }

                                env.rethrowKotlinException {
                                    CallVoidMethodA!!(env, gListener, mOnThemeChangedListenerOnChanged.get(env), args)
                                }
                            }
                        }
                    }
                }

                override fun onExited() {
                    globalJavaVM.withAttachedThread { env ->
                        env.functions {
                            memScoped {
                                val args: CArrayPointer<jvalue> = jValues(0)

                                env.rethrowKotlinException {
                                    CallVoidMethodA!!(env, gListener, mOnThemeChangedListenerOnExited.get(env), args)
                                }
                            }
                        }
                    }
                }

                override fun onError(e: Exception) {
                    globalJavaVM.withAttachedThread { env ->
                        env.functions {
                            memScoped {
                                val args: CArrayPointer<jvalue> = jValues(1) {
                                    it[0].l = e.toJavaException(env)
                                }

                                env.rethrowKotlinException {
                                    CallVoidMethodA!!(env, gListener, mOnThemeChangedListenerOnError.get(env), args)
                                }
                            }
                        }
                    }
                }

                override fun onFinalize() {
                    globalJavaVM.withAttachedThread { env ->
                        env.functions {
                            DeleteGlobalRef!!(env, gListener)
                        }
                    }
                }
            }

            val disposable = ThemeCompat.addListener(kListener)

            StableRef.create(disposable).asCPointer().toLong()
        }
    } ?: 0L
}

@CName("Java_com_github_kr328_clash_compat_ThemeCompat_nativeDisposeListener")
fun nativeDisposeListener(env: CPointer<JNIEnvVar>, clazz: jclass, disposable: jlong) {
    disposable.toCPointer<CPointed>()!!.asStableRef<Disposable>().get().dispose()
}

@CName("Java_com_github_kr328_clash_compat_ThemeCompat_nativeReleaseListener")
fun nativeReleaseListener(env: CPointer<JNIEnvVar>, clazz: jclass, disposable: jlong) {
    disposable.toCPointer<CPointed>()!!.asStableRef<Disposable>().dispose()
}
