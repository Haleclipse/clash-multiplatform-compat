package com.github.kr328.clash.compat;

import org.jetbrains.annotations.NotNull;

import java.io.IOException;

public final class NotificationCompat {
    static {
        CompatLibrary.load();
    }

    private static native boolean nativeIsSupported();

    public static boolean isSupported() {
        return nativeIsSupported();
    }

    private static native void nativeSendNotification(
            @NotNull final String applicationId,
            @NotNull final String title,
            @NotNull final String message
    ) throws IOException;

    public static void sendNotification(
            @NotNull final String applicationId,
            @NotNull final String title,
            @NotNull final String message
    ) throws IOException {
        nativeSendNotification(applicationId, title, message);
    }
}
