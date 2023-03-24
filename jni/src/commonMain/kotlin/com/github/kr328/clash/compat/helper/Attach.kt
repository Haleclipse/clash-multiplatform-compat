package com.github.kr328.clash.compat.helper

import kotlinx.cinterop.*
import java.JNIEnvVar
import java.JNI_OK
import java.JNI_VERSION_1_8
import java.JavaVMVar

fun <R> CPointer<JavaVMVar>.withAttachedThread(block: (CPointer<JNIEnvVar>) -> R): R {
    return memScoped {
        functions { vm ->
            val env: CPointerVar<JNIEnvVar> = alloc()

            if (GetEnv!!(vm, env.ptr.reinterpret(), JNI_VERSION_1_8) == JNI_OK) {
                block(env.value!!)
            } else {
                if (AttachCurrentThread!!(vm, env.ptr.reinterpret(), null) != JNI_OK) {
                    throw JavaException("Unable to attach current thread")
                }

                try {
                    block(env.value!!)
                } finally {
                    DetachCurrentThread!!(vm)
                }
            }
        }
    }
}
