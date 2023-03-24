package com.github.kr328.clash.compat;

import java.io.Closeable;

public final class Utils {
    public static void closeSilent(final Closeable closeable) {
        try {
            closeable.close();
        } catch (final Exception ignored) {
            // ignored
        }
    }
}

