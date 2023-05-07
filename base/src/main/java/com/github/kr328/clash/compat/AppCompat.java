package com.github.kr328.clash.compat;

public final class AppCompat {
    static {
        CompatLibrary.load();
    }

    private static native void nativeSetProcessApplicationID(final String applicationID);

    public static void setProcessApplicationID(final String applicationID) {
        nativeSetProcessApplicationID(applicationID);
    }
}
