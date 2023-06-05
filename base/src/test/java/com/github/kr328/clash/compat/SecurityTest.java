package com.github.kr328.clash.compat;

import com.sun.security.auth.module.UnixSystem;
import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.Assumptions;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

public class SecurityTest {
    @Test
    void getUidGid() {
        Assumptions.assumeTrue(System.getProperty("os.name").toLowerCase().contains("linux"));

        final UnixSystem system = new UnixSystem();

        Assertions.assertEquals(system.getUid(), SecurityCompat.getUnixUid());
        Assertions.assertEquals(system.getGid(), SecurityCompat.getUnixGid());
    }

    @Test
    void getSELinuxContext() throws IOException {
        String currentContext;
        try {
            currentContext = new String(Files.readAllBytes(Path.of("/proc/self/attr/current")));
            if (currentContext.endsWith("\0")) {
                currentContext = currentContext.substring(0, currentContext.length() - 1);
            }
        } catch (final IOException e) {
            currentContext = null;
        }

        Assumptions.assumeTrue(currentContext != null);

        Assertions.assertEquals(currentContext, SecurityCompat.getSELinuxContext());
    }
}
