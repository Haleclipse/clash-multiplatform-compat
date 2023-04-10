package com.github.kr328.clash.compat;

import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.Assumptions;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Comparator;
import java.util.stream.Stream;

public class LoadTest {
    @Test
    void load() {
        CompatLibrary.load();
    }

    @Test
    void setOverrideExtractPath() throws IOException {
        Assumptions.assumeTrue(System.getenv("COMPAT_LIBRARY_PATH") == null);

        final Path extractPath = Path.of("build", "compat-library");

        try (final Stream<Path> files = Files.walk(extractPath)) {
            for (final Path file : files.sorted(Comparator.reverseOrder()).toList()) {
                Files.delete(file);
            }
        } catch (final IOException e) {
            // ignored
        }

        CompatLibrary.setOverrideExtractPath(extractPath);

        CompatLibrary.load();

        try (final Stream<Path> files = Files.list(extractPath)) {
            Assertions.assertTrue(files.map(Object::toString).anyMatch((p) -> p.endsWith(".so") || p.endsWith(".dll")));
        }
    }
}
