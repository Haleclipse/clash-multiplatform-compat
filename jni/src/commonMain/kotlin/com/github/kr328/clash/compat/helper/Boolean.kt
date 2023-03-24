package com.github.kr328.clash.compat.helper

import java.jboolean

fun jboolean.asBoolean(): Boolean {
    return this != 0.toUByte()
}
