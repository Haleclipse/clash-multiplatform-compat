@file:Suppress("unused", "unused_parameter")

package com.github.kr328.clash.compat

import com.github.kr328.clash.compat.helper.rethrowJavaException
import kotlinx.cinterop.CPointer
import java.JNIEnvVar
import java.jclass
import java.jint
import java.jlong

enum class WindowFrame {
    EDGE_INSETS,
    TITLE_BAR,
}

enum class WindowControl {
    CLOSE_BUTTON,
    BACK_BUTTON,
}

data class Rectangle(val left: Int, val top: Int, val right: Int, val bottom: Int) {
    companion object {
        val Zero: Rectangle = Rectangle(0, 0, 0, 0)
    }
}

value class Point private constructor(private val packed: Long) {
    constructor(x: Int, y: Int) : this((x.toLong() shl 32) or y.toLong())

    val x: Int
        get() = (packed ushr 32).toInt()

    val y: Int
        get() = (packed and 0xffff).toInt()
}

operator fun Rectangle.contains(point: Point): Boolean {
    return (point.x in left until right) and (point.y in top until bottom)
}

expect object WindowCompat {
    fun setBorderless(window: FileDescriptor)
    fun setFrameSize(window: FileDescriptor, frame: WindowFrame, size: Int)
    fun setControlPosition(window: FileDescriptor, control: WindowControl, rectangle: Rectangle)
}

@CName("Java_com_github_kr328_clash_compat_WindowCompat_nativeSetBorderless")
fun nativeSetBorderless(env: CPointer<JNIEnvVar>, clazz: jclass, window: jlong) {
    env.rethrowJavaException {
        WindowCompat.setBorderless(window)
    }
}

@CName("Java_com_github_kr328_clash_compat_WindowCompat_nativeSetFrameSize")
fun nativeSetFrameSize(env: CPointer<JNIEnvVar>, clazz: jclass, window: jlong, frame: jint, size: jint) {
    env.rethrowJavaException {
        WindowCompat.setFrameSize(window, WindowFrame.values()[frame], size)
    }
}

@CName("Java_com_github_kr328_clash_compat_WindowCompat_nativeSetControlPosition")
fun nativeSetControlPosition(
    env: CPointer<JNIEnvVar>,
    clazz: jclass,
    window: jlong,
    control: jint,
    left: jint,
    top: jint,
    right: jint,
    bottom: jint,
) {
    env.rethrowJavaException {
        WindowCompat.setControlPosition(
            window,
            WindowControl.values()[control],
            Rectangle(left, top, right, bottom),
        )
    }
}
