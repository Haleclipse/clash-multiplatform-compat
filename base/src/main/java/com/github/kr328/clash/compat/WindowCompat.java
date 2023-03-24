package com.github.kr328.clash.compat;

import org.jetbrains.annotations.NotNull;

import java.util.Objects;

public final class WindowCompat {
    static {
        CompatLibrary.load();
    }

    private static native void nativeSetBorderless(long handle);

    public static void setBorderless(final long handle) {
        nativeSetBorderless(handle);
    }

    private static native void nativeSetFrameSize(long handle, int frame, int size);

    public static void setFrameSize(final long handle, @NotNull final WindowFrame frame, final int size) {
        nativeSetFrameSize(handle, Objects.requireNonNull(frame).ordinal(), size);
    }

    private static native void nativeSetControlPosition(long handle, int control, int left, int top, int right, int bottom);

    public static void setControlPosition(final long handle, @NotNull final WindowControl control, final int left, final int top, final int right, final int bottom) {
        nativeSetControlPosition(handle, Objects.requireNonNull(control).ordinal(), left, top, right, bottom);
    }

    public enum WindowFrame {
        EDGE_INSETS,
        TITLE_BAR,
    }

    public enum WindowControl {
        CLOSE_BUTTON,
        BACK_BUTTON
    }
}
