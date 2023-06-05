package com.github.kr328.clash.compat;

import java.io.IOException;

public class SecurityCompat {
    static {
        CompatLibrary.load();
    }

    private static native int nativeGetUnixUid();

    public static int getUnixUid() {
        return nativeGetUnixUid();
    }

    private static native int nativeGetUnixGid();

    public static int getUnixGid() {
        return nativeGetUnixGid();
    }

    private static native String nativeGetSELinuxContext() throws IOException;

    public static String getSELinuxContext() throws IOException {
        return nativeGetSELinuxContext();
    }
}
