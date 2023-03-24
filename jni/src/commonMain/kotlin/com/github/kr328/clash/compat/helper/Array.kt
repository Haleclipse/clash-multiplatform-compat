package com.github.kr328.clash.compat.helper

import java.JNIEnvVar
import java.jobject
import java.jobjectArray
import kotlinx.cinterop.CPointer
import kotlinx.cinterop.invoke

fun jobjectArray.toList(env: CPointer<JNIEnvVar>): List<jobject?> {
    return env.functions {
        val length = env.rethrowKotlinException {
            GetArrayLength!!(env, this@toList)
        }

        List(length) {
            env.rethrowKotlinException {
                GetObjectArrayElement!!(env, this@toList, it)
            }
        }
    }
}
