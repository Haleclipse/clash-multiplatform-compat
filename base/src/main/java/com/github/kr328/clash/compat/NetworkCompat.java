package com.github.kr328.clash.compat;

import org.jetbrains.annotations.NotNull;

import java.io.IOException;
import java.util.List;

public final class NetworkCompat {
    static {
        CompatLibrary.load();
    }

    private static native boolean nativeIsSystemProxySupported();

    public static boolean isSystemProxySupported() {
        return nativeIsSystemProxySupported();
    }

    private static native void nativeSetSystemProxy(
            final boolean enabled,
            @NotNull final String address,
            @NotNull final String @NotNull [] excludes
    ) throws IOException;

    public static void setSystemProxy(
            final boolean enabled,
            @NotNull final String address,
            @NotNull final List<String> excludes
    ) throws IOException {
        nativeSetSystemProxy(enabled, address, excludes.toArray(String[]::new));
    }
}
