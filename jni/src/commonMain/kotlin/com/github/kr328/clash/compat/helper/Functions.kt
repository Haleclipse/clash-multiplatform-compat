package com.github.kr328.clash.compat.helper

import kotlinx.cinterop.CPointer
import kotlinx.cinterop.pointed
import java.JNIEnvVar
import java.JNIInvokeInterface_
import java.JNINativeInterface_
import java.JavaVMVar

inline fun <R> CPointer<JavaVMVar>.functions(block: JNIInvokeInterface_.(CPointer<JavaVMVar>) -> R): R {
    return pointed.pointed!!.block(this)
}

inline fun <R> CPointer<JNIEnvVar>.functions(block: JNINativeInterface_.(CPointer<JNIEnvVar>) -> R): R {
    return pointed.pointed!!.block(this)
}
