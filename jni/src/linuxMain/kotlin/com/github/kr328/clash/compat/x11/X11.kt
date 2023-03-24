package com.github.kr328.clash.compat.x11

import com.github.kr328.clash.compat.SyscallException
import kotlinx.cinterop.CPointer
import kotlinx.cinterop.CValue
import kotlinx.cinterop.cValue
import linux.*

inline fun <R> withDisplay(block: (CPointer<Display>) -> R): R {
    val display = XOpenDisplay(null) ?: throw SyscallException("XOpenDisplay failed")

    return try {
        block(display)
    } finally {
        XCloseDisplay(display)
    }
}

inline fun xClientMessage(
    display: CPointer<Display>,
    window: Window,
    messageType: String,
    data: XClientMessageEvent.() -> Unit,
): CValue<XEvent> {
    return cValue {
        xclient.type = ClientMessage
        xclient.display = display
        xclient.window = window
        xclient.message_type = XInternAtom(display, messageType, True)
        xclient.data()
    }
}
