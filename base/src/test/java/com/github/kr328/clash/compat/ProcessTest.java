package com.github.kr328.clash.compat;

import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.Test;

import java.io.ByteArrayOutputStream;
import java.nio.file.Path;
import java.util.List;
import java.util.concurrent.CompletableFuture;

public class ProcessTest {
    @Test
    void createProcess() throws Exception {
        final Path pingPath;
        final List<String> arguments;
        if (System.getProperty("os.name").toLowerCase().contains("win")) {
            pingPath = Path.of("C:\\Windows\\System32\\ping.exe");
            arguments = List.of("ping.exe", "-n", "4", "127.0.0.1");
        } else {
            pingPath = Path.of("/usr/bin/ping");
            arguments = List.of("ping", "-c", "4", "127.0.0.1");
        }

        try (final FileCompat.Pipe stdout = FileCompat.createPipe()) {
            FileCompat.setFileDescriptorInheritable(stdout.writer().getFD(), true);

            final ProcessCompat.Process process = ProcessCompat.createProcess(
                    pingPath,
                    arguments,
                    null,
                    null,
                    null,
                    stdout.writer().getFD(),
                    null,
                    List.of(stdout.writer().getFD())
            );
            try (process) {
                stdout.writer().close();

                final ByteArrayOutputStream output = new ByteArrayOutputStream();
                stdout.reader().transferTo(output);

                Assertions.assertNotEquals(0, output.size());

                final CompletableFuture<Integer> result = new CompletableFuture<>();
                process.getResult().handle((ret, throwable) -> {
                    if (throwable != null) {
                        result.completeExceptionally(throwable);
                    } else {
                        result.complete(ret);
                    }

                    return null;
                });

                Assertions.assertEquals(0, result.join(), output::toString);
            }
        }
    }
}
