package com.github.kr328.clash.compat;

import org.jetbrains.annotations.NotNull;

import java.io.IOException;
import java.lang.ref.Cleaner;

public final class ThemeCompat {
    private static final Cleaner holderCleaner = Cleaner.create();

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

    private static native void nativeReleaseListener(long ptr);

    @NotNull
    public static Disposable addListener(@NotNull final OnThemeChangedListener listener) throws IOException {
        return new Holder(nativeAddListener(listener));
    }

    public interface Disposable {
        void dispose();
    }

    @SuppressWarnings("unused")
    public interface OnThemeChangedListener {
        void onChanged(final boolean isNight);
    }

    private static class Holder implements Disposable {
        private final Cleaner.Cleanable cleanable;

        public Holder(final long ptr) {
            this.cleanable = holderCleaner.register(this, () -> nativeReleaseListener(ptr));
        }

        @Override
        public void dispose() {
            cleanable.clean();
        }
    }
}
