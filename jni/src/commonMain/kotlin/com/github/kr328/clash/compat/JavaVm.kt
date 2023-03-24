@file:Suppress("unused", "unused_parameter")

package com.github.kr328.clash.compat

import kotlinx.cinterop.COpaquePointer
import kotlinx.cinterop.CPointer
import java.JNI_VERSION_1_8
import java.JavaVMVar
import java.jint

lateinit var globalJavaVM: CPointer<JavaVMVar>
    private set

@CName("JNI_OnLoad")
fun nativeOnLoad(vm: CPointer<JavaVMVar>, reversed: COpaquePointer?): jint {
    globalJavaVM = vm

    WindowCompat //.init()

    return JNI_VERSION_1_8
}
