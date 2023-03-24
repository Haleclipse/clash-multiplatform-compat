package com.github.kr328.clash.compat.helper

import java.jvalue
import kotlinx.cinterop.CArrayPointer
import kotlinx.cinterop.NativePlacement
import kotlinx.cinterop.allocArray

inline fun NativePlacement.jValues(
    size: Int,
    initializer: (CArrayPointer<jvalue>) -> Unit = {}
): CArrayPointer<jvalue> {
    return allocArray<jvalue>(size).apply(initializer)
}
