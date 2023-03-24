package com.github.kr328.clash.compat.helper

import kotlinx.cinterop.*
import java.JNIEnvVar
import java.jstring
import java.jvalue

private val oStandardCharsetsUTF8 = LazyStaticFieldObject(
    cfg = LazyField(
        cfg = LazyMemberConfig(
            className = "java/nio/charset/StandardCharsets",
            name = "UTF_8",
            signature = "Ljava/nio/charset/Charset;",
            isStatic = true,
        ),
    )
)

private val mStringGetBytes: LazyMethod = LazyMethod(
    cfg = LazyMemberConfig(
        className = "java/lang/String",
        name = "getBytes",
        signature = "(Ljava/nio/charset/Charset;)[B",
        isStatic = false,
    ),
)

fun jstring.toKString(env: CPointer<JNIEnvVar>): String {
    return memScoped {
        env.functions {
            env.suppressException {
                val array = CallObjectMethodA!!(
                    env,
                    this@toKString,
                    mStringGetBytes.get(env),
                    jValues(1) { it[0].l = oStandardCharsetsUTF8.get(env) },
                )

                ByteArray(GetArrayLength!!(env, array)).also {
                    it.usePinned { bytes ->
                        GetByteArrayRegion!!(env, array, 0, bytes.get().size, bytes.addressOf(0))
                    }
                }.toKString()
            }
        }
    }
}

private val mStringNew = LazyMethod(
    cfg = LazyMemberConfig(
        className = "java/lang/String",
        name = "<init>",
        signature = "([BLjava/nio/charset/Charset;)V",
        isStatic = false,
    ),
)

private val cString = LazyClass(
    className = "java/lang/String"
)

fun String.toJString(env: CPointer<JNIEnvVar>): jstring {
    return memScoped {
        env.functions {
            env.suppressException {
                val bytes = encodeToByteArray().usePinned {
                    NewByteArray!!(env, it.get().size).also { array ->
                        SetByteArrayRegion!!(env, array, 0, it.get().size, it.addressOf(0))
                    }
                }

                val args: CArrayPointer<jvalue> = jValues(2) {
                    it[0].l = bytes
                    it[1].l = oStandardCharsetsUTF8.get(env)
                }

                NewObjectA!!(env, cString.get(env), mStringNew.get(env), args)!!
            }
        }
    }
}
