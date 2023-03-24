package com.github.kr328.clash.compat

import com.github.kr328.clash.compat.os.syscall
import kotlinx.cinterop.*
import windows.*

actual object WindowCompat {
    private class WindowContext(val root: HWND) {
        var position: Rectangle = Rectangle(0, 0, 0, 0)
        val frameSizes: MutableList<Int> = MutableList(WindowFrame.values().size) { 0 }
        val controlPositions: MutableList<Rectangle> = MutableList(WindowControl.values().size) { Rectangle.Zero }

        fun matchArea(worldPoint: Point): Int {
            val position = position
            val localPoint = Point(worldPoint.x - position.left, worldPoint.y - position.top)

            if (controlPositions.any { localPoint in it }) {
                return HTCLIENT
            }

            val width: Int = position.right - position.left
            val height: Int = position.bottom - position.top
            val edgeInset: Int = frameSizes[WindowFrame.EDGE_INSETS.ordinal]

            val x = localPoint.x
            val y = localPoint.y
            val inLeft: Boolean = x < edgeInset
            val inTop: Boolean = y < edgeInset
            val inRight: Boolean = x > width - edgeInset
            val inBottom: Boolean = y > height - edgeInset

            return when {
                inTop && inLeft -> HTTOPLEFT
                inTop && inRight -> HTTOPRIGHT
                inTop -> HTTOP
                inBottom && inLeft -> HTBOTTOMLEFT
                inBottom && inRight -> HTBOTTOMRIGHT
                inBottom -> HTBOTTOM
                inLeft -> HTLEFT
                inRight -> HTRIGHT
                else -> {
                    return if (y < frameSizes[WindowFrame.TITLE_BAR.ordinal]) {
                        HTCAPTION
                    } else {
                        HTCLIENT
                    }
                }
            }
        }
    }

    private var HWND.context: StableRef<WindowContext>?
        get() {
            return GetPropA(this, "compat-context")?.asStableRef()
        }
        set(value) {
            SetPropA(this, "compat-context", value?.asCPointer())
        }

    private var HWND.awtProcedure: WNDPROC?
        get() {
            return GetPropA(this, "awt-procedure")?.reinterpret()
        }
        set(value) {
            SetPropA(this, "awt-procedure", value?.reinterpret())
        }

    private fun getCaptionPadding(handle: HWND): Int {
        return if (IsZoomed(handle) == TRUE) {
            GetSystemMetricsForDpi(SM_CXPADDEDBORDER, GetDpiForWindow(handle))
        } else {
            0
        }
    }

    private fun delegatedWindowProcedure(
        handle: HWND,
        message: UINT,
        wParam: WPARAM,
        lParam: LPARAM
    ): LRESULT {
        val awtProcedure = handle.awtProcedure
            ?: return DefWindowProcA(handle, message, wParam, lParam)
        val context = handle.context?.get()
            ?: return CallWindowProcA(awtProcedure, handle, message, wParam, lParam)

        when (message.toInt()) {
            WM_DESTROY -> {
                val ref = handle.context

                handle.context = null
                handle.awtProcedure = null

                ref?.dispose()
            }
            WM_NCHITTEST -> {
                val area = context.matchArea(Point(getXFromLParam(lParam), getYFromLParam(lParam)))
                if (context.root == handle) {
                    return area.toLong()
                }

                if (area != HTCLIENT) {
                    return HTTRANSPARENT.toLong()
                }

                return area.toLong()
            }
            WM_NCCALCSIZE -> {
                if (context.root == handle && wParam != 0uL) {
                    val params: CPointer<NCCALCSIZE_PARAMS>? = lParam.toCPointer()
                    if (params != null) {
                        params.pointed.rgrc[0].top += getCaptionPadding(handle)

                        return 0
                    }
                }
            }
            WM_SIZE, WM_MOVE -> {
                if (context.root == handle) {
                    memScoped {
                        val rect: RECT = alloc()

                        GetWindowRect(handle, rect.ptr)

                        rect.top += getCaptionPadding(handle)

                        context.position = Rectangle(rect.left, rect.top, rect.right, rect.bottom)
                    }
                }
            }
            WM_NCRBUTTONDOWN -> {
                if (wParam.toInt() == HTCAPTION) {
                    return 0
                }
            }
            WM_NCRBUTTONUP -> {
                if (wParam.toInt() == HTCAPTION) {
                    val x = getXFromLParam(lParam)
                    val y = getYFromLParam(lParam)

                    val menu = GetSystemMenu(handle, FALSE)

                    TrackPopupMenu(menu, 0, x, y, 0, context.root, null)

                    return 0
                }
            }
            WM_COMMAND -> {
                if ((wParam and 0xf000u) != 0uL) {
                    return SendMessageA(handle, WM_SYSCOMMAND, wParam, lParam)
                }
            }
            WM_SYSCOMMAND -> {
                return DefWindowProcA(context.root, message, wParam, lParam)
            }
        }

        return CallWindowProcA(awtProcedure, handle, message, wParam, lParam)
    }


    private fun attachToWindow(handle: HWND, lParam: LPARAM): BOOL {
        if (handle == INVALID_HANDLE_VALUE) {
            return FALSE
        }

        if (handle.context != null) {
            return TRUE
        }

        val context = lParam.toCPointer<CPointed>()!!.asStableRef<WindowContext>().get()

        handle.context = StableRef.create(context)
        handle.awtProcedure = GetWindowLongPtrA(handle, GWLP_WNDPROC).toCPointer()

        SetWindowLongPtrA(
            handle,
            GWLP_WNDPROC,
            staticCFunction { h: HWND, m: UINT, w: WPARAM, l: LPARAM -> delegatedWindowProcedure(h, m, w, l) }.toLong()
        )

        EnumChildWindows(
            handle,
            staticCFunction { h: HWND, p: LPARAM -> attachToWindow(h, p) }.reinterpret(),
            lParam
        )

        SetWindowPos(
            handle,
            null,
            0,
            0,
            0,
            0,
            (SWP_FRAMECHANGED or SWP_NOMOVE or SWP_NOSIZE or SWP_NOZORDER).toUInt(),
        )

        return TRUE
    }

    actual fun setBorderless(window: FileDescriptor) {
        val margins: CValue<MARGINS> = cValue {
            cxLeftWidth = 0
            cyTopHeight = 0
            cxRightWidth = 0
            cyBottomHeight = 1
        }

        syscall("DwmExtendFrameIntoClientArea") {
            DwmExtendFrameIntoClientArea(window.toCPointer(), margins)
        }

        syscall("SetWindowLongA") {
            SetWindowLongA(
                window.toCPointer(),
                GWL_STYLE,
                WS_OVERLAPPEDWINDOW,
            )
        }

        val context = WindowContext(window.toCPointer()!!)

        val contextRef = StableRef.create(context)
        try {
            attachToWindow(context.root, contextRef.asCPointer().toLong())
        } finally {
            contextRef.dispose()
        }
    }

    actual fun setFrameSize(window: FileDescriptor, frame: WindowFrame, size: Int) {
        val handle: HWND = window.toCPointer()!!

        handle.context?.get()?.frameSizes?.set(frame.ordinal, size)
    }

    actual fun setControlPosition(window: FileDescriptor, control: WindowControl, rectangle: Rectangle) {
        val handle: HWND = window.toCPointer()!!

        handle.context?.get()?.controlPositions?.set(control.ordinal, rectangle)
    }
}
