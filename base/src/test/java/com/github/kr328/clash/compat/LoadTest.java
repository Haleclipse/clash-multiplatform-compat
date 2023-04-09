package com.github.kr328.clash.compat;

import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.Assumptions;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.stream.Stream;

public class LoadTest {
    @Test
    void load() {
        CompatLibrary.load();
    }

    @Test
    void setOverrideExtractPath() throws IOException {
        Assumptions.assumeTrue(System.getenv("COMPAT_LIBRARY_PATH") == null);

        final Path tempDir = Files.createTempDirectory("compat-test");

        CompatLibrary.setOverrideExtractPath(tempDir);

        CompatLibrary.load();

        try (final Stream<Path> files = Files.list(tempDir)) {
            Assertions.assertTrue(files.peek(System.out::println).anyMatch((p) -> p.endsWith(".so") || p.endsWith(".dll")));
        }
    }
}
