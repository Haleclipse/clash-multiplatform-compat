package com.github.kr328.clash.compat

import com.github.kr328.clash.compat.concurrent.Mutex
import com.github.kr328.clash.compat.concurrent.withLock
import com.github.kr328.clash.compat.helper.functions
import com.github.kr328.clash.compat.helper.rethrowKotlinException
import com.github.kr328.clash.compat.helper.withAttachedThread
import com.github.kr328.clash.compat.x11.withDisplay
import com.github.kr328.clash.compat.x11.xClientMessage
import kotlinx.cinterop.*
import linux.*
import java.JNIEnvVar
import java.JNINativeMethod
import java.jclass
import java.jlong

actual object WindowCompat {
    private class WindowContext(val appRoot: Window) {
        private var width: Int = 0
        private var height: Int = 0

        private val controlPositions: MutableList<Rectangle> = MutableList(WindowControl.values().size) { Rectangle.Zero }
        private val frameSizes: MutableList<Int> = MutableList(WindowFrame.values().size) { 0 }

        operator fun set(idx: WindowControl, value: Rectangle) {
            controlPositions[idx.ordinal] = value
        }

        operator fun set(idx: WindowFrame, value: Int) {
            frameSizes[idx.ordinal] = value
        }

        fun resize(width: Int, height: Int) {
            this.width = width
            this.height = height
        }

        fun matchesTitleBar(point: Point): Boolean {
            if (controlPositions.any { point in it }) {
                return false
            }

            val insets = frameSizes[WindowFrame.EDGE_INSETS.ordinal]
            return point in Rectangle(insets, insets, width - insets, frameSizes[WindowFrame.TITLE_BAR.ordinal])
        }
    }

    private val mutex = Mutex()
    private val windows = mutableMapOf<Window, WindowContext>()

    private fun findWindowContext(window: Window): WindowContext? {
        return mutex.withLock {
            windows[window]
        }
    }

    private fun removeWindow(window: Window) {
        mutex.withLock {
            windows.remove(window)
        }
    }

    private fun delegatedXNextEvent(display: CPointer<Display>, event: CPointer<XEvent>) {
        while (true) {
            XNextEvent(display, event)

            when (event.pointed.type) {
                DestroyNotify -> {
                    removeWindow(event.pointed.xdestroywindow.window)
                }
                ConfigureNotify -> {
                    findWindowContext(event.pointed.xconfigure.window)?.apply {
                        resize(event.pointed.xconfigure.width, event.pointed.xconfigure.height)
                    }
                }
                ButtonPress -> {
                    when (event.pointed.xbutton.button.toInt()) {
                        Button1 -> {
                            val consumed: Boolean? = findWindowContext(event.pointed.xbutton.window)?.run {
                                if (matchesTitleBar(Point(event.pointed.xbutton.x, event.pointed.xbutton.y))) {
                                    val request: CValue<XEvent> = xClientMessage(
                                        display,
                                        appRoot,
                                        "_NET_WM_MOVERESIZE",
                                    ) {
                                        format = 32
                                        data.l[0] = event.pointed.xbutton.x_root.toLong()
                                        data.l[1] = event.pointed.xbutton.y_root.toLong()
                                        data.l[2] = 8 // _NET_WM_MOVERESIZE_MOVE
                                        data.l[3] = Button1.toLong()
                                        data.l[4] = 1 // normal applications
                                    }

                                    XSendEvent(
                                        display,
                                        XDefaultRootWindow(display),
                                        False,
                                        SubstructureNotifyMask or SubstructureRedirectMask,
                                        request
                                    )

                                    true
                                } else {
                                    false
                                }
                            }
                            if (consumed == true) {
                                continue
                            }
                        }
                        Button3 -> {
                            val matches = findWindowContext(event.pointed.xbutton.window)
                                ?.matchesTitleBar(Point(event.pointed.xbutton.x, event.pointed.xbutton.y))
                            if (matches == true) {
                                continue
                            }
                        }
                    }
                }
                ButtonRelease -> {
                    when (event.pointed.xbutton.button.toInt()) {
                        Button1 -> {
                            val consumed: Boolean? = findWindowContext(event.pointed.xbutton.window)
                                ?.matchesTitleBar(Point(event.pointed.xbutton.x, event.pointed.xbutton.y))
                            if (consumed == true) {
                                continue
                            }
                        }
                        Button3 -> {
                            val consumed: Boolean? = findWindowContext(event.pointed.xbutton.window)?.run {
                                if (matchesTitleBar(Point(event.pointed.xbutton.x, event.pointed.xbutton.y))) {
                                    val request: CValue<XEvent> = xClientMessage(
                                        display,
                                        appRoot,
                                        "_GTK_SHOW_WINDOW_MENU",
                                    ) {
                                        format = 32
                                        data.l[0] = 0
                                        data.l[1] = event.pointed.xbutton.x_root.toLong()
                                        data.l[2] = event.pointed.xbutton.y_root.toLong()
                                    }

                                    XSendEvent(
                                        display,
                                        XDefaultRootWindow(display),
                                        False,
                                        SubstructureNotifyMask or SubstructureRedirectMask,
                                        request
                                    )

                                    true
                                } else {
                                    false
                                }
                            }
                            if (consumed != null) {
                                continue
                            }
                        }
                    }
                }
            }

            return
        }
    }

    init {
        globalJavaVM.withAttachedThread { env ->
            memScoped {
                env.functions {
                    val clazz = env.rethrowKotlinException {
                        FindClass!!(env, "sun/awt/X11/XlibWrapper".cstr.ptr)
                    }!!

                    val functions: CArrayPointer<JNINativeMethod> = allocArray<JNINativeMethod>(1).also {
                        it[0].apply {
                            name = "XNextEvent".cstr.ptr
                            signature = "(JJ)V".cstr.ptr
                            fnPtr = staticCFunction { _: CPointer<JNIEnvVar>, _: jclass, display: jlong, event: jlong ->
                                delegatedXNextEvent(display.toCPointer()!!, event.toCPointer()!!)
                            }
                        }
                    }

                    env.rethrowKotlinException {
                        RegisterNatives!!(env, clazz, functions, 1)
                    }
                }
            }
        }
    }

    private fun storeWindow(display: CPointer<Display>, window: Window, context: WindowContext) {
        if (window in windows) {
            return
        }

        windows[window] = context

        memScoped {
            val root: WindowVar = alloc()
            val parent: WindowVar = alloc()
            val children: CArrayPointerVar<WindowVar> = alloc()
            val childrenLength: UIntVar = alloc()

            XQueryTree(display, window, root.ptr, parent.ptr, children.ptr, childrenLength.ptr)
            if (children.value != null) {
                repeat(childrenLength.value.toInt()) {
                    storeWindow(display, children.value!![it], context)
                }

                XFree(children.value)
            }
        }
    }

    actual fun setBorderless(window: FileDescriptor) {
        mutex.withLock {
            withDisplay { display ->
                storeWindow(display, window.toULong(), WindowContext(window.toULong()))
            }
        }
    }

    actual fun setFrameSize(window: FileDescriptor, frame: WindowFrame, size: Int) {
        findWindowContext(window.toULong())?.also {
            it[frame] = size
        }
    }

    actual fun setControlPosition(window: FileDescriptor, control: WindowControl, rectangle: Rectangle) {
        findWindowContext(window.toULong())?.also {
            it[control] = rectangle
        }
    }
}
