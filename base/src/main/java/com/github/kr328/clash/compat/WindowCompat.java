package com.github.kr328.clash.compat;

import org.jetbrains.annotations.NotNull;

import java.io.IOException;
import java.lang.ref.Cleaner;

public final class WindowCompat {
    private static final Cleaner contextCleaner = Cleaner.create();

    static {
        CompatLibrary.load();
    }

    private static native boolean nativeIsSupported();

    public static boolean isSupported() {
        return nativeIsSupported();
    }

    private static native long nativeSetBorderless(long handle) throws IOException;

    @NotNull
    public static Context setBorderless(final long handle) throws IOException {
        return new Context(nativeSetBorderless(handle));
    }

    private static native void nativeContextSetFrameSize(long ptr, int frame, int size);

    private static native void nativeContextSetControlPosition(long ptr, int control, int left, int top, int right, int bottom);

    private static native void nativeContextRelease(long ptr);

    public enum WindowFrame {
        EDGE_INSETS,
        TITLE_BAR,
    }

    public enum WindowControl {
        CLOSE_BUTTON,
        BACK_BUTTON,
        MINIMIZE_BUTTON,
    }

    public static class Context {
        private final long ptr;

        Context(final long ptr) {
            this.ptr = ptr;

            contextCleaner.register(this, () -> nativeContextRelease(ptr));
        }

        public void setFrameSize(@NotNull final WindowFrame frame, final int size) {
            nativeContextSetFrameSize(ptr, frame.ordinal(), size);
        }

        public void setControlPosition(@NotNull final WindowControl control, final int left, final int top, final int right, final int bottom) {
            nativeContextSetControlPosition(ptr, control.ordinal(), left, top, right, bottom);
        }
    }
}
