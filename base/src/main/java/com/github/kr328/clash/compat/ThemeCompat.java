package com.github.kr328.clash.compat;

import org.jetbrains.annotations.NotNull;

import java.io.IOException;
import java.lang.ref.Cleaner;

public final class ThemeCompat {
    private static final Cleaner cleaner = Cleaner.create();

    static {
        CompatLibrary.load();
    }

    private static native boolean nativeIsSupported();

    public static boolean isSupported() {
        return nativeIsSupported();
    }

    private static native boolean nativeIsNight() throws IOException;

    public static boolean isNight() throws IOException {
        return nativeIsNight();
    }

    private static native long nativeAddListener(@NotNull OnThemeChangedListener listener) throws IOException;

    private static native void nativeDisposeListener(long ptr);

    private static native void nativeReleaseListener(long ptr);

    @NotNull
    public static Disposable addListener(
            @NotNull final OnThemeChangedListener listener
    ) throws IOException {
        final long ptr = nativeAddListener(listener);

        final Disposable disposable = () -> nativeDisposeListener(ptr);

        cleaner.register(disposable, () -> nativeReleaseListener(ptr));

        return disposable;
    }

    public interface Disposable {
        void dispose();
    }

    public interface OnThemeChangedListener {
        void onChanged(final boolean isNight);

        void onExited();

        void onError(final Exception e);
    }
}
