package com.github.kr328.clash.compat;

import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.Assumptions;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.util.List;

public class NetworkTest {
    @Test
    public void addRemoveSystemProxy() throws IOException {
        Assumptions.assumeTrue(NetworkCompat.isSystemProxySupported());

        NetworkCompat.setSystemProxy(true, "127.0.0.1:8080", List.of("127.0.0.1", "localhost"));

        Assertions.assertTrue(Window.showIsSuccessWindow("Set System Proxy"));

        NetworkCompat.setSystemProxy(false, "", List.of());

        Assertions.assertTrue(Window.showIsSuccessWindow("Remove System Proxy"));
    }
}
